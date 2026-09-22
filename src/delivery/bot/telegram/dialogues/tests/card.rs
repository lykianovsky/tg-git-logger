use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::tests::TelegramBotTestsAction;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::utils::builder::message::MessageBuilder;
use rust_i18n::t;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Карточка последнего прогона: один и тот же вид для команды, обновления и итогов
pub fn build_card_text(run: Option<&TestRun>) -> String {
    let Some(run) = run else {
        return t!("telegram_bot.dialogues.tests.no_runs").to_string();
    };

    let status_key = format!("report.test_run.status.{}", run.status.as_str());
    let totals = run.totals.unwrap_or_default();

    let mut builder = MessageBuilder::new()
        .bold(&t!("telegram_bot.dialogues.tests.title").to_string())
        .empty_line()
        .with_html_escape(true)
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

    builder.build()
}

pub fn build_card_keyboard(run: Option<&TestRun>) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    if let Some(run) = run {
        // Пока прогон идёт, итогов и отчёта ещё нет — показываем только обновление
        if run.is_active() {
            rows.push(vec![button(TelegramBotTestsAction::Refresh)]);
            rows.push(vec![button(TelegramBotTestsAction::Close)]);

            return InlineKeyboardMarkup::new(rows);
        }

        let mut result_row = vec![button(TelegramBotTestsAction::Report)];

        if run.has_failures() {
            result_row.push(button(TelegramBotTestsAction::Failures));
        }

        rows.push(result_row);
    }

    rows.push(vec![
        button(TelegramBotTestsAction::RunAll),
        button(TelegramBotTestsAction::ChooseBlock),
    ]);
    rows.push(vec![
        button(TelegramBotTestsAction::Refresh),
        button(TelegramBotTestsAction::Close),
    ]);

    InlineKeyboardMarkup::new(rows)
}

fn button(action: TelegramBotTestsAction) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(action.label(), action.to_callback_data().to_string())
}
