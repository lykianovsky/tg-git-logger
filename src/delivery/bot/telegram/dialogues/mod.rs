use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::bind_repository::TelegramBotBindRepositoryState;
use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationState;
use crate::delivery::bot::telegram::dialogues::report::TelegramBotDialogueReportByDateRangeState;
use crate::delivery::bot::telegram::dialogues::setup_webhook::TelegramBotSetupWebhookState;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

pub mod admin;
pub mod bind_repository;
pub mod registration;
pub mod report;
pub mod setup_webhook;

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueState {
    #[default]
    Idle,
    Registration(TelegramBotDialogueRegistrationState),
    ReportByDateRange(TelegramBotDialogueReportByDateRangeState),
    Admin(TelegramBotDialogueAdminState),
    BindRepository(TelegramBotBindRepositoryState),
    SetupWebhook(TelegramBotSetupWebhookState),
}

pub type TelegramBotDialogueType =
    Dialogue<TelegramBotDialogueState, InMemStorage<TelegramBotDialogueState>>;
