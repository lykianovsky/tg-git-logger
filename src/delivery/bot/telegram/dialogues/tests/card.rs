use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::tests::TelegramBotTestsAction;
use crate::domain::notification::services::notification_service::{
    NotificationButton, NotificationKeyboard,
};
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::ports::test_runner::TestRunProgress;
use crate::utils::builder::message::MessageBuilder;
use rust_i18n::t;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Длина полосы прогресса в символах
const PROGRESS_BAR_WIDTH: u32 = 10;

/// Полоса из символов: в мессенджере это единственный способ показать ход прогона
pub fn build_progress_bar(progress: &TestRunProgress) -> String {
    if progress.total_steps == 0 {
        return String::new();
    }

    let filled = (progress.completed_steps * PROGRESS_BAR_WIDTH / progress.total_steps)
        .min(PROGRESS_BAR_WIDTH);

    format!(
        "{}{} {}/{}",
        "▓".repeat(filled as usize),
        "░".repeat((PROGRESS_BAR_WIDTH - filled) as usize),
        progress.completed_steps,
        progress.total_steps
    )
}

/// Карточка последнего прогона: один и тот же вид для команды, обновления и итогов
pub fn build_card_text(
    run: Option<&TestRun>,
    is_configured: bool,
    repository_title: &str,
    progress: Option<&TestRunProgress>,
) -> String {
    // Карточка всегда называет репозиторий: у пользователя их может быть несколько
    let header = MessageBuilder::new()
        .bold(&t!("telegram_bot.dialogues.tests.title").to_string())
        .with_html_escape(true)
        .line(repository_title)
        .empty_line();

    if !is_configured {
        return header
            .line(&t!("telegram_bot.dialogues.tests.not_connected").to_string())
            .build();
    }

    let Some(run) = run else {
        return header
            .line(&t!("telegram_bot.dialogues.tests.no_runs").to_string())
            .build();
    };

    let status_key = format!("report.test_run.status.{}", run.status.as_str());
    let totals = run.totals.unwrap_or_default();

    let mut builder = header
        .section(
            &t!("telegram_bot.dialogues.tests.status").to_string(),
            &t!(&status_key).to_string(),
        )
        .section_code(
            &t!("telegram_bot.test_run.branch").to_string(),
            &run.git_ref,
        );

    if let Some(args) = run.args.as_deref() {
        builder = builder.section_code(&t!("telegram_bot.dialogues.tests.scope").to_string(), args);
    }

    if run.totals.is_some() {
        builder = builder.section(
            &t!("telegram_bot.test_run.totals").to_string(),
            &t!(
                "telegram_bot.test_run.totals_value",
                passed = totals.passed,
                failed = totals.failed,
                flaky = totals.flaky,
                skipped = totals.skipped
            )
            .to_string(),
        );
    }

    if let Some(finished_at) = run.finished_at {
        builder = builder.section(
            &t!("telegram_bot.dialogues.tests.finished").to_string(),
            &finished_at.format("%d.%m.%Y, %H:%M UTC").to_string(),
        );
    }

    // Пока прогон идёт, показываем, где он сейчас
    if let Some(progress) = progress.filter(|_| run.is_active()) {
        builder = builder.section(
            &t!("telegram_bot.dialogues.tests.progress").to_string(),
            &build_progress_bar(progress),
        );

        if let Some(current_step) = progress.current_step.as_deref() {
            builder = builder.section(
                &t!("telegram_bot.dialogues.tests.current_step").to_string(),
                current_step,
            );
        }
    }

    if run.is_active() {
        let elapsed = chrono::Utc::now() - run.started_at.unwrap_or(run.created_at);

        builder = builder.section(
            &t!("telegram_bot.dialogues.tests.elapsed").to_string(),
            &t!(
                "telegram_bot.dialogues.tests.elapsed_value",
                minutes = elapsed.num_minutes().max(0),
                seconds = (elapsed.num_seconds().max(0)) % 60
            )
            .to_string(),
        );
    }

    if let Some(run_url) = run.run_url.as_deref() {
        builder = builder
            .empty_line()
            .link(&t!("report.test_run.run_in_ci").to_string(), run_url);
    }

    builder.build()
}

pub fn build_card_keyboard(run: Option<&TestRun>, is_configured: bool) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        card_actions(run, is_configured)
            .into_iter()
            .map(|row| row.into_iter().map(button).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
    )
}

/// Та же клавиатура для автообновления карточки: иначе кнопки пропадут
pub fn build_card_notification_keyboard(
    run: Option<&TestRun>,
    is_configured: bool,
) -> NotificationKeyboard {
    card_actions(run, is_configured)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|action| NotificationButton {
                    label: action.label().to_string(),
                    action: action.to_callback_data().to_string(),
                })
                .collect()
        })
        .collect()
}

fn card_actions(run: Option<&TestRun>, is_configured: bool) -> Vec<Vec<TelegramBotTestsAction>> {
    let mut rows: Vec<Vec<TelegramBotTestsAction>> = Vec::new();

    // Пока тесты не подключены, запускать нечего — предлагаем настройку
    if !is_configured {
        return vec![
            vec![TelegramBotTestsAction::Connect],
            vec![TelegramBotTestsAction::Close],
        ];
    }

    if let Some(run) = run {
        // Пока прогон идёт, итогов и отчёта ещё нет — показываем только обновление
        if run.is_active() {
            return vec![
                vec![
                    TelegramBotTestsAction::Refresh,
                    TelegramBotTestsAction::CancelRun,
                ],
                vec![TelegramBotTestsAction::Close],
            ];
        }

        let mut result_row = vec![TelegramBotTestsAction::Report];

        if run.has_failures() {
            result_row.push(TelegramBotTestsAction::Failures);
        }

        rows.push(result_row);

        // Гонять весь набор ради двух упавших незачем
        if run.has_failures() {
            rows.push(vec![TelegramBotTestsAction::RerunFailed]);
        }
    }

    rows.push(vec![
        TelegramBotTestsAction::RunAll,
        TelegramBotTestsAction::ChooseBlock,
    ]);
    rows.push(vec![
        TelegramBotTestsAction::Readiness,
        TelegramBotTestsAction::Dashboard,
    ]);
    rows.push(vec![
        TelegramBotTestsAction::Refresh,
        TelegramBotTestsAction::Close,
    ]);

    rows
}

fn button(action: TelegramBotTestsAction) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(action.label(), action.to_callback_data().to_string())
}
