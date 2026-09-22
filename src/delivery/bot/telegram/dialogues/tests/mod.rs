pub mod card;

use crate::application::test_run::commands::dispatch_test_run::command::DispatchTestRunCommand;
use crate::application::test_run::commands::dispatch_test_run::error::DispatchTestRunError;
use crate::application::test_run::queries::build_test_report::query::BuildTestReportQuery;
use crate::application::test_run::queries::get_last_test_run::query::GetLastTestRunQuery;
use crate::application::test_run::queries::get_run_failures::query::GetRunFailuresQuery;
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
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::utils::builder::message::MessageBuilder;
use rust_i18n::t;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{Handler, case};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{Requester, Update};
use teloxide::types::{
    CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode,
};
use teloxide::{Bot, dptree};

/// Кнопка запуска блока несёт его путь, поэтому у неё свой префикс —
/// иначе путь не отличить от имени действия
const BLOCK_CALLBACK_PREFIX: &str = "block:";

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
}

pub struct TelegramBotTestsDispatcher {}

impl TelegramBotTestsDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        Update::filter_callback_query()
            .branch(case![TelegramBotTestsState::SelectRepository].endpoint(choose_repository))
            .branch(case![TelegramBotTestsState::Card { repository_id }].endpoint(handle_card))
            .branch(
                case![TelegramBotTestsState::SelectBlock { repository_id }].endpoint(choose_block),
            )
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
    let run = executors
        .queries
        .get_last_test_run
        .execute(&GetLastTestRunQuery { repository_id })
        .await
        .map(|response| response.run)
        .unwrap_or_default();

    bot.edit_message_text(chat_id, message_id, build_card_text(run.as_ref()))
        .parse_mode(ParseMode::Html)
        .reply_markup(build_card_keyboard(run.as_ref()))
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

    let Ok(action) =
        TelegramBotTestsAction::from_callback_data(query.data.as_deref().unwrap_or(""))
    else {
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let blocks = match executors
        .queries
        .list_test_blocks
        .execute(&ListTestBlocksQuery {
            repository_id: RepositoryId(repository_id),
        })
        .await
    {
        Ok(response) => response.blocks,
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = executors
        .commands
        .dispatch_test_run
        .execute(&DispatchTestRunCommand {
            repository_id: RepositoryId(repository_id),
            git_ref: None,
            args,
            trigger: TestRunTrigger::Chat,
            requested_by_user_id: None,
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

    bot.edit_message_text(chat_id, message_id, builder.build())
        .parse_mode(ParseMode::Html)
        .reply_markup(back_keyboard())
        .await?;

    Ok(())
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
