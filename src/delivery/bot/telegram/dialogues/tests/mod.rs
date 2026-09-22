pub mod card;

use crate::application::task::queries::list_task_tracker_options::error::ListTaskTrackerOptionsError;
use crate::application::task::queries::list_task_tracker_options::query::ListTaskTrackerOptionsQuery;
use crate::application::task::queries::list_task_tracker_options::response::TaskTrackerOption;
use crate::application::test_run::commands::connect_test_suite::command::ConnectTestSuiteCommand;
use crate::application::test_run::commands::create_test_failure_card::command::CreateTestFailureCardCommand;
use crate::application::test_run::commands::create_test_failure_card::error::CreateTestFailureCardError;
use crate::application::test_run::commands::create_test_failure_card::response::CreateTestFailureCardResponse;
use crate::application::test_run::commands::dispatch_test_run::command::DispatchTestRunCommand;
use crate::application::test_run::commands::dispatch_test_run::error::DispatchTestRunError;
use crate::application::test_run::queries::build_test_report::query::BuildTestReportQuery;
use crate::application::test_run::queries::get_last_test_run::query::GetLastTestRunQuery;
use crate::application::test_run::queries::get_run_failures::query::GetRunFailuresQuery;
use crate::application::test_run::queries::list_ci_options::error::ListCiOptionsError;
use crate::application::test_run::queries::list_ci_options::query::{
    CiOptionKind, ListCiOptionsQuery,
};
use crate::application::test_run::queries::list_test_blocks::error::ListTestBlocksError;
use crate::application::test_run::queries::list_test_blocks::query::ListTestBlocksQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::tests::card::{
    build_card_keyboard, build_card_text,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::tests::TelegramBotTestsAction;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::ports::test_runner::CiOption;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use rust_i18n::t;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{Handler, case};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{Requester, Update};
use teloxide::types::{
    CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
    ParseMode,
};
use teloxide::{Bot, dptree};

/// Кнопка запуска блока несёт его путь, поэтому у неё свой префикс —
/// иначе путь не отличить от имени действия
const BLOCK_CALLBACK_PREFIX: &str = "block:";
/// Кнопка заведения карточки несёт идентификатор упавшего теста
const CARD_CALLBACK_PREFIX: &str = "card:";
/// Кнопка выбора человека несёт его идентификатор в трекере
const OPTION_CALLBACK_PREFIX: &str = "opt:";
/// Кнопка тега несёт его название: именно им тег вешается на карточку
const TAG_CALLBACK_PREFIX: &str = "tag:";
/// Сколько вариантов показываем на шаге выбора
const MAX_OPTION_BUTTONS: usize = 30;

#[derive(Debug, Clone, Default)]
pub enum TelegramBotTestsState {
    #[default]
    SelectRepository,

    Card {
        repository_id: i32,
    },

    SelectBlock {
        repository_id: i32,
    },

    /// Подключение тестов: шаг мастера и всё, что уже выбрано
    Connect {
        step: ConnectStep,
        draft: ConnectDraft,
    },

    /// Шаг 1 формы карточки: на кого её повесить
    SelectCardAssignee {
        repository_id: i32,
        test_failure_id: i32,
    },

    /// Шаг 2: каким тегом пометить
    SelectCardTag {
        repository_id: i32,
        test_failure_id: i32,
        responsible_id: u64,
    },
}

/// Шаги подключения. Куда класть карточки по упавшим тестам, бот берёт из настроек
/// трекера репозитория — второй раз спейс и колонку не спрашиваем
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectStep {
    Workflow,
    Branch,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectDraft {
    pub repository_id: i32,
    pub workflow_file: String,
}

pub struct TelegramBotTestsDispatcher {}

impl TelegramBotTestsDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let callbacks = Update::filter_callback_query()
            .branch(case![TelegramBotTestsState::SelectRepository].endpoint(choose_repository))
            .branch(case![TelegramBotTestsState::Card { repository_id }].endpoint(handle_card))
            .branch(
                case![TelegramBotTestsState::SelectBlock { repository_id }].endpoint(choose_block),
            )
            .branch(
                case![TelegramBotTestsState::SelectCardAssignee {
                    repository_id,
                    test_failure_id
                }]
                .endpoint(choose_card_assignee),
            )
            .branch(
                case![TelegramBotTestsState::SelectCardTag {
                    repository_id,
                    test_failure_id,
                    responsible_id
                }]
                .endpoint(choose_card_tag),
            )
            .branch(case![TelegramBotTestsState::Connect { step, draft }].endpoint(handle_connect));

        dptree::entry().branch(callbacks)
    }
}

/// Карточку перерисовываем на месте: в чате остаётся одно сообщение о тестах
pub async fn render_card(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: RepositoryId,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = executors
        .queries
        .get_last_test_run
        .execute(&GetLastTestRunQuery { repository_id })
        .await;

    let (run, is_configured, repository_title) = match response {
        Ok(response) => (
            response.run,
            response.is_configured,
            response.repository_title,
        ),
        Err(error) => {
            tracing::error!(%error, "Failed to load last test run");

            (None, true, String::new())
        }
    };

    bot.edit_message_text(
        chat_id,
        message_id,
        build_card_text(run.as_ref(), is_configured, &repository_title),
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(build_card_keyboard(run.as_ref(), is_configured))
    .await?;

    Ok(())
}

async fn choose_repository(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let Ok(repository_id) = query.data.as_deref().unwrap_or("").parse::<i32>() else {
        return Ok(());
    };

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::Card { repository_id },
        ))
        .await?;

    render_card(
        &bot,
        &executors,
        chat_id,
        message_id,
        RepositoryId(repository_id),
    )
    .await
}

async fn handle_card(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    repository_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let data = query.data.as_deref().unwrap_or("");

    if let Some(raw_failure_id) = data.strip_prefix(CARD_CALLBACK_PREFIX) {
        let Ok(test_failure_id) = raw_failure_id.parse::<i32>() else {
            return Ok(());
        };

        return ask_card_assignee(
            &bot,
            &executors,
            &dialogue,
            chat_id,
            message_id,
            repository_id,
            test_failure_id,
        )
        .await;
    }

    let Ok(action) = TelegramBotTestsAction::from_callback_data(data) else {
        return Ok(());
    };

    match action {
        TelegramBotTestsAction::Refresh => {
            render_card(
                &bot,
                &executors,
                chat_id,
                message_id,
                RepositoryId(repository_id),
            )
            .await?
        }

        TelegramBotTestsAction::Report => {
            show_report(&bot, &executors, chat_id, message_id, repository_id).await?
        }

        TelegramBotTestsAction::Failures => {
            show_failures(&bot, &executors, chat_id, message_id, repository_id).await?
        }

        TelegramBotTestsAction::RunAll => {
            run_tests(
                &bot,
                &executors,
                chat_id,
                message_id,
                repository_id,
                String::new(),
                social_user_id,
            )
            .await?
        }

        TelegramBotTestsAction::ChooseBlock => {
            show_blocks(
                &bot,
                &executors,
                &dialogue,
                chat_id,
                message_id,
                repository_id,
                social_user_id,
            )
            .await?
        }

        TelegramBotTestsAction::Connect => {
            start_connect(
                &bot,
                &executors,
                &dialogue,
                chat_id,
                message_id,
                repository_id,
                social_user_id,
            )
            .await?
        }

        TelegramBotTestsAction::Back => {
            render_card(
                &bot,
                &executors,
                chat_id,
                message_id,
                RepositoryId(repository_id),
            )
            .await?
        }

        TelegramBotTestsAction::Close => {
            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.closed").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

            dialogue.exit().await.ok();
        }
    }

    Ok(())
}

async fn choose_block(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    repository_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let data = query.data.as_deref().unwrap_or("");

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::Card { repository_id },
        ))
        .await?;

    let Some(block) = data.strip_prefix(BLOCK_CALLBACK_PREFIX) else {
        return render_card(
            &bot,
            &executors,
            chat_id,
            message_id,
            RepositoryId(repository_id),
        )
        .await;
    };

    run_tests(
        &bot,
        &executors,
        chat_id,
        message_id,
        repository_id,
        block.to_string(),
        SocialUserId(query.from.id.0 as i32),
    )
    .await
}

async fn show_blocks(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    dialogue: &TelegramBotDialogueType,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let blocks = match executors
        .queries
        .list_test_blocks
        .execute(&ListTestBlocksQuery {
            repository_id: RepositoryId(repository_id),
            social_user_id,
        })
        .await
    {
        Ok(response) => response.blocks,
        Err(ListTestBlocksError::NoVersionControlAccount) => {
            return edit_with_back(
                bot,
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.no_github_account").to_string(),
            )
            .await;
        }
        Err(error) => {
            tracing::error!(%error, "Failed to list test blocks");

            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.blocks_error").to_string(),
            )
            .reply_markup(back_keyboard())
            .await?;

            return Ok(());
        }
    };

    // Каталога тестов в ветке нет — запускать по блокам нечего
    if blocks.is_empty() {
        return edit_with_back(
            bot,
            chat_id,
            message_id,
            t!("telegram_bot.dialogues.tests.no_blocks").to_string(),
        )
        .await;
    }

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::SelectBlock { repository_id },
        ))
        .await?;

    let mut rows: Vec<Vec<InlineKeyboardButton>> = blocks
        .iter()
        .map(|block| {
            vec![InlineKeyboardButton::callback(
                block.clone(),
                format!("{BLOCK_CALLBACK_PREFIX}{block}"),
            )]
        })
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        TelegramBotTestsAction::Back.label(),
        TelegramBotTestsAction::Back.to_callback_data().to_string(),
    )]);

    bot.edit_message_text(
        chat_id,
        message_id,
        t!("telegram_bot.dialogues.tests.select_block").to_string(),
    )
    .reply_markup(InlineKeyboardMarkup::new(rows))
    .await?;

    Ok(())
}

async fn run_tests(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
    args: String,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = executors
        .commands
        .dispatch_test_run
        .execute(&DispatchTestRunCommand {
            repository_id: RepositoryId(repository_id),
            git_ref: None,
            args,
            trigger: TestRunTrigger::Chat,
            requested_by_social_user_id: Some(social_user_id),
            chat_id: Some(SocialChatId(chat_id.0)),
        })
        .await;

    let text = match result {
        Ok(response) => started_text(&response.run),
        Err(DispatchTestRunError::AlreadyRunning(run)) => t!(
            "telegram_bot.dialogues.tests.already_running",
            branch = MessageBuilder::escape_html(&run.git_ref)
        )
        .to_string(),
        Err(DispatchTestRunError::NotConfigured) => {
            t!("telegram_bot.dialogues.tests.not_configured").to_string()
        }
        // Без привязанного GitHub запускать нечем: прогон идёт правами пользователя
        Err(DispatchTestRunError::NoVersionControlAccount) => {
            t!("telegram_bot.dialogues.tests.no_github_account").to_string()
        }
        Err(error) => {
            tracing::error!(%error, "Failed to dispatch test run");

            t!("telegram_bot.dialogues.tests.dispatch_error").to_string()
        }
    };

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(refresh_keyboard())
        .await?;

    Ok(())
}

fn started_text(run: &TestRun) -> String {
    let scope = match run.args.as_deref() {
        Some(args) if !args.is_empty() => args.to_string(),
        _ => t!("telegram_bot.dialogues.tests.all_tests").to_string(),
    };

    MessageBuilder::new()
        .bold(&t!("telegram_bot.dialogues.tests.started").to_string())
        .empty_line()
        .with_html_escape(true)
        .section_code(
            &t!("telegram_bot.test_run.branch").to_string(),
            &run.git_ref,
        )
        .section_code(
            &t!("telegram_bot.dialogues.tests.scope").to_string(),
            &scope,
        )
        .build()
}

async fn show_report(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(run) = last_run(executors, repository_id).await else {
        return Ok(());
    };

    let text = match executors
        .queries
        .build_test_report
        .execute(&BuildTestReportQuery {
            test_run_id: run.id,
        })
        .await
    {
        Ok(response) => MessageBuilder::new()
            .bold(&t!("telegram_bot.dialogues.tests.report_ready").to_string())
            .empty_line()
            .link(
                &t!("telegram_bot.test_run.report_link").to_string(),
                &response.report_url,
            )
            .build(),
        Err(error) => {
            tracing::error!(%error, "Failed to build test report");

            t!("telegram_bot.dialogues.tests.report_error").to_string()
        }
    };

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(back_keyboard())
        .await?;

    Ok(())
}

async fn show_failures(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(run) = last_run(executors, repository_id).await else {
        return Ok(());
    };

    let failures = executors
        .queries
        .get_run_failures
        .execute(&GetRunFailuresQuery {
            test_run_id: run.id,
        })
        .await
        .map(|response| response.failures)
        .unwrap_or_default();

    let mut builder = MessageBuilder::new()
        .bold(
            &t!(
                "telegram_bot.dialogues.tests.failures_title",
                count = failures.len()
            )
            .to_string(),
        )
        .empty_line()
        .with_html_escape(true);

    for item in &failures {
        builder = builder.section_bold(&item.failure.project, &item.failure.title);
        builder = builder.code(&item.failure.file);

        if let Some(error) = item.failure.error_excerpt.as_deref() {
            builder = builder.italic(error);
        }

        if let Some(card) = item.card.as_ref() {
            builder = builder.link(
                &t!("report.test_run.card_exists").to_string(),
                &card.card_url,
            );
        }

        builder = builder.empty_line();
    }

    let mut rows: Vec<Vec<InlineKeyboardButton>> = failures
        .iter()
        .filter(|item| item.card.is_none())
        .map(|item| {
            vec![InlineKeyboardButton::callback(
                t!(
                    "telegram_bot.dialogues.tests.create_card",
                    title = &item.failure.title
                )
                .to_string(),
                format!("{CARD_CALLBACK_PREFIX}{}", item.failure.id),
            )]
        })
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        TelegramBotTestsAction::Back.label(),
        TelegramBotTestsAction::Back.to_callback_data().to_string(),
    )]);

    bot.edit_message_text(chat_id, message_id, builder.build())
        .parse_mode(ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::new(rows))
        .await?;

    Ok(())
}

/// Подключение начинается со списка процессов CI репозитория — руками ничего не вводим
async fn start_connect(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    dialogue: &TelegramBotDialogueType,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let options = match load_ci_options(
        executors,
        repository_id,
        social_user_id,
        CiOptionKind::Workflows,
    )
    .await
    {
        Ok(options) => options,
        Err(error) => return show_ci_options_error(bot, chat_id, message_id, error).await,
    };

    // В репозитории нет ни одного процесса CI — подключать нечего
    if options.is_empty() {
        return edit_with_back(
            bot,
            chat_id,
            message_id,
            t!("telegram_bot.dialogues.tests.no_workflows").to_string(),
        )
        .await;
    }

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::Connect {
                step: ConnectStep::Workflow,
                draft: ConnectDraft {
                    repository_id,
                    workflow_file: String::new(),
                },
            },
        ))
        .await?;

    bot.edit_message_text(
        chat_id,
        message_id,
        t!("telegram_bot.dialogues.tests.select_workflow").to_string(),
    )
    .reply_markup(ci_options_keyboard(&options))
    .await?;

    Ok(())
}

async fn handle_connect(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (step, draft): (ConnectStep, ConnectDraft),
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let data = query.data.as_deref().unwrap_or("");

    if data == TelegramBotTestsAction::Back.to_callback_data() {
        return back_to_card(
            &bot,
            &executors,
            &dialogue,
            chat_id,
            message_id,
            draft.repository_id,
        )
        .await;
    }

    let Some(value) = data.strip_prefix(OPTION_CALLBACK_PREFIX) else {
        return Ok(());
    };

    match step {
        ConnectStep::Workflow => {
            let options = match load_ci_options(
                &executors,
                draft.repository_id,
                social_user_id,
                CiOptionKind::Branches,
            )
            .await
            {
                Ok(options) => options,
                Err(error) => {
                    return show_ci_options_error(&bot, chat_id, message_id, error).await;
                }
            };

            if options.is_empty() {
                return edit_with_back(
                    &bot,
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.tests.no_branches").to_string(),
                )
                .await;
            }

            dialogue
                .update(TelegramBotDialogueState::Tests(
                    TelegramBotTestsState::Connect {
                        step: ConnectStep::Branch,
                        draft: ConnectDraft {
                            repository_id: draft.repository_id,
                            workflow_file: value.to_string(),
                        },
                    },
                ))
                .await?;

            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.select_branch").to_string(),
            )
            .reply_markup(ci_options_keyboard(&options))
            .await?;
        }

        ConnectStep::Branch => {
            let connected = executors
                .commands
                .connect_test_suite
                .execute(&ConnectTestSuiteCommand {
                    repository_id: RepositoryId(draft.repository_id),
                    workflow_file: draft.workflow_file.clone(),
                    default_ref: value.to_string(),
                })
                .await;

            if let Err(error) = connected {
                tracing::error!(%error, "Failed to connect test suite");

                return edit_with_back(
                    &bot,
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.tests.connect_error").to_string(),
                )
                .await;
            }

            back_to_card(
                &bot,
                &executors,
                &dialogue,
                chat_id,
                message_id,
                draft.repository_id,
            )
            .await?;
        }
    }

    Ok(())
}

async fn load_ci_options(
    executors: &Arc<ApplicationBoostrapExecutors>,
    repository_id: i32,
    social_user_id: SocialUserId,
    kind: CiOptionKind,
) -> Result<Vec<CiOption>, ListCiOptionsError> {
    executors
        .queries
        .list_ci_options
        .execute(&ListCiOptionsQuery {
            repository_id: RepositoryId(repository_id),
            social_user_id,
            kind,
        })
        .await
        .map(|response| response.options)
}

async fn show_ci_options_error(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    error: ListCiOptionsError,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = match error {
        ListCiOptionsError::NoVersionControlAccount => {
            t!("telegram_bot.dialogues.tests.no_github_account").to_string()
        }
        error => {
            tracing::error!(%error, "Failed to load CI options");

            t!("telegram_bot.dialogues.tests.ci_options_error").to_string()
        }
    };

    edit_with_back(bot, chat_id, message_id, text).await
}

fn ci_options_keyboard(options: &[CiOption]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .take(MAX_OPTION_BUTTONS)
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.label.clone(),
                format!("{OPTION_CALLBACK_PREFIX}{}", option.value),
            )]
        })
        .collect();

    rows.push(vec![back_button()]);

    InlineKeyboardMarkup::new(rows)
}

/// Шаг 1: список людей из трекера. Ничего не зашиваем — состав команды меняется
async fn ask_card_assignee(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    dialogue: &TelegramBotDialogueType,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
    test_failure_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let options = match load_tracker_options(executors, ListTaskTrackerOptionsQuery::Users).await {
        Ok(options) => options,
        Err(error) => {
            tracing::error!(%error, "Failed to load task tracker users");

            return edit_with_back(
                bot,
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.tracker_options_error").to_string(),
            )
            .await;
        }
    };

    if options.is_empty() {
        return edit_with_back(
            bot,
            chat_id,
            message_id,
            t!("telegram_bot.dialogues.tests.no_tracker_users").to_string(),
        )
        .await;
    }

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::SelectCardAssignee {
                repository_id,
                test_failure_id,
            },
        ))
        .await?;

    bot.edit_message_text(
        chat_id,
        message_id,
        t!("telegram_bot.dialogues.tests.select_assignee").to_string(),
    )
    .reply_markup(options_keyboard(&options))
    .await?;

    Ok(())
}

async fn choose_card_assignee(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (repository_id, test_failure_id): (i32, i32),
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let data = query.data.as_deref().unwrap_or("");

    if data == TelegramBotTestsAction::Back.to_callback_data() {
        return back_to_card(
            &bot,
            &executors,
            &dialogue,
            chat_id,
            message_id,
            repository_id,
        )
        .await;
    }

    let Some(responsible_id) = data
        .strip_prefix(OPTION_CALLBACK_PREFIX)
        .and_then(|raw| raw.parse::<u64>().ok())
    else {
        return Ok(());
    };

    let options = match load_tracker_options(&executors, ListTaskTrackerOptionsQuery::Tags).await {
        Ok(options) => options,
        Err(error) => {
            tracing::error!(%error, "Failed to load task tracker tags");

            return edit_with_back(
                &bot,
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.tests.tracker_options_error").to_string(),
            )
            .await;
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::SelectCardTag {
                repository_id,
                test_failure_id,
                responsible_id,
            },
        ))
        .await?;

    bot.edit_message_text(
        chat_id,
        message_id,
        t!("telegram_bot.dialogues.tests.select_tag").to_string(),
    )
    .reply_markup(tag_keyboard(&options))
    .await?;

    Ok(())
}

async fn choose_card_tag(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (repository_id, test_failure_id, responsible_id): (i32, i32, u64),
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let Some((chat_id, message_id)) = message_target(&query) else {
        return Ok(());
    };

    let data = query.data.as_deref().unwrap_or("");

    if data == TelegramBotTestsAction::Back.to_callback_data() {
        return back_to_card(
            &bot,
            &executors,
            &dialogue,
            chat_id,
            message_id,
            repository_id,
        )
        .await;
    }

    // Тег необязателен: карточку можно завести и без него
    let tag = data
        .strip_prefix(TAG_CALLBACK_PREFIX)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string);

    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::Card { repository_id },
        ))
        .await?;

    create_failure_card(
        &bot,
        &executors,
        chat_id,
        message_id,
        test_failure_id,
        responsible_id,
        tag,
    )
    .await
}

async fn create_failure_card(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    test_failure_id: i32,
    responsible_id: u64,
    tag: Option<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = executors
        .commands
        .create_test_failure_card
        .execute(&CreateTestFailureCardCommand {
            test_failure_id,
            responsible_id: Some(responsible_id),
            tag,
            created_by_user_id: None,
        })
        .await;

    let text = match result {
        Ok(CreateTestFailureCardResponse::Created(card)) => MessageBuilder::new()
            .bold(&t!("telegram_bot.dialogues.tests.card_created").to_string())
            .empty_line()
            .link(&card.title, &card.card_url)
            .build(),
        // Карточка по этому тесту уже есть — показываем её, вторую не заводим
        Ok(CreateTestFailureCardResponse::AlreadyExists(card)) => MessageBuilder::new()
            .bold(&t!("telegram_bot.dialogues.tests.card_already_exists").to_string())
            .empty_line()
            .link(&card.title, &card.card_url)
            .build(),
        Err(CreateTestFailureCardError::TrackerNotConfigured) => {
            t!("telegram_bot.dialogues.tests.tracker_not_configured").to_string()
        }
        Err(error) => {
            tracing::error!(%error, "Failed to create test failure card");

            t!("telegram_bot.dialogues.tests.card_error").to_string()
        }
    };

    bot.edit_message_text(chat_id, message_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(back_keyboard())
        .await?;

    Ok(())
}

async fn load_tracker_options(
    executors: &Arc<ApplicationBoostrapExecutors>,
    query: ListTaskTrackerOptionsQuery,
) -> Result<Vec<TaskTrackerOption>, ListTaskTrackerOptionsError> {
    executors
        .queries
        .list_task_tracker_options
        .execute(&query)
        .await
        .map(|response| response.options)
}

async fn back_to_card(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    dialogue: &TelegramBotDialogueType,
    chat_id: ChatId,
    message_id: MessageId,
    repository_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    dialogue
        .update(TelegramBotDialogueState::Tests(
            TelegramBotTestsState::Card { repository_id },
        ))
        .await?;

    render_card(
        bot,
        executors,
        chat_id,
        message_id,
        RepositoryId(repository_id),
    )
    .await
}

async fn edit_with_back(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    text: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.edit_message_text(chat_id, message_id, text)
        .reply_markup(back_keyboard())
        .await?;

    Ok(())
}

/// Людей в трекере много — режем список, иначе клавиатура не влезет в сообщение
fn options_keyboard(options: &[TaskTrackerOption]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .take(MAX_OPTION_BUTTONS)
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.name.clone(),
                format!("{OPTION_CALLBACK_PREFIX}{}", option.id),
            )]
        })
        .collect();

    rows.push(vec![back_button()]);

    InlineKeyboardMarkup::new(rows)
}

fn tag_keyboard(options: &[TaskTrackerOption]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .take(MAX_OPTION_BUTTONS)
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.name.clone(),
                format!("{TAG_CALLBACK_PREFIX}{}", option.name),
            )]
        })
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        t!("telegram_bot.dialogues.tests.without_tag").to_string(),
        TAG_CALLBACK_PREFIX.to_string(),
    )]);
    rows.push(vec![back_button()]);

    InlineKeyboardMarkup::new(rows)
}

fn back_button() -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        TelegramBotTestsAction::Back.label(),
        TelegramBotTestsAction::Back.to_callback_data().to_string(),
    )
}

async fn last_run(
    executors: &Arc<ApplicationBoostrapExecutors>,
    repository_id: i32,
) -> Option<TestRun> {
    executors
        .queries
        .get_last_test_run
        .execute(&GetLastTestRunQuery {
            repository_id: RepositoryId(repository_id),
        })
        .await
        .ok()
        .and_then(|response| response.run)
}

fn back_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        TelegramBotTestsAction::Back.label(),
        TelegramBotTestsAction::Back.to_callback_data().to_string(),
    )]])
}

fn refresh_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        TelegramBotTestsAction::Refresh.label(),
        TelegramBotTestsAction::Refresh
            .to_callback_data()
            .to_string(),
    )]])
}

fn message_target(query: &CallbackQuery) -> Option<(ChatId, MessageId)> {
    let message = query.message.as_ref()?;

    Some((message.chat().id, message.id()))
}
