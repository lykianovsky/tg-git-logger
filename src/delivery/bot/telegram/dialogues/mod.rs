use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::bind_repository::TelegramBotBindRepositoryState;
use crate::delivery::bot::telegram::dialogues::digest::TelegramBotDigestState;
use crate::delivery::bot::telegram::dialogues::notifications::TelegramBotNotificationsState;
use crate::delivery::bot::telegram::dialogues::onboarding::TelegramBotOnboardingState;
use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationState;
use crate::delivery::bot::telegram::dialogues::release_plan::TelegramBotReleasePlanState;
use crate::delivery::bot::telegram::dialogues::report::TelegramBotDialogueReportByDateRangeState;
use crate::delivery::bot::telegram::dialogues::setup_notifications::TelegramBotSetupNotificationsState;
use crate::delivery::bot::telegram::dialogues::setup_webhook::TelegramBotSetupWebhookState;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

pub mod admin;
pub mod bind_repository;
pub mod digest;
pub mod helpers;
pub mod notifications;
pub mod onboarding;
pub mod registration;
pub mod release_plan;
pub mod report;
pub mod setup_notifications;
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
    SetupNotifications(TelegramBotSetupNotificationsState),
    Digest(TelegramBotDigestState),
    Notifications(TelegramBotNotificationsState),
    Onboarding(TelegramBotOnboardingState),
    ReleasePlan(TelegramBotReleasePlanState),
}

pub type TelegramBotDialogueType =
    Dialogue<TelegramBotDialogueState, InMemStorage<TelegramBotDialogueState>>;
