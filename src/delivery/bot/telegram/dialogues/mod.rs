use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationState;
use crate::delivery::bot::telegram::dialogues::report::TelegramBotDialogueReportByDateRangeState;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

pub mod registration;
pub mod report;

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueState {
    #[default]
    Idle,
    Registration(TelegramBotDialogueRegistrationState),
    ReportByDateRange(TelegramBotDialogueReportByDateRangeState),
}

pub type TelegramBotDialogueType =
    Dialogue<TelegramBotDialogueState, InMemStorage<TelegramBotDialogueState>>;
