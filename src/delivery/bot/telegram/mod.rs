mod commands;
pub mod context;
pub mod dialogues;
mod keyboards;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminDispatcher;
use crate::delivery::bot::telegram::dialogues::bind_repository::TelegramBotBindRepositoryDispatcher;
use crate::delivery::bot::telegram::dialogues::digest::TelegramBotDigestDispatcher;
use crate::delivery::bot::telegram::dialogues::notifications::TelegramBotNotificationsDispatcher;
use crate::delivery::bot::telegram::dialogues::onboarding::TelegramBotOnboardingDispatcher;
use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationDispatcher;
use crate::delivery::bot::telegram::dialogues::release_plan::TelegramBotReleasePlanDispatcher;
use crate::delivery::bot::telegram::dialogues::release_plan_settings::TelegramBotReleasePlanSettingsDispatcher;
use crate::delivery::bot::telegram::dialogues::report::TelegramBotDialogueReportByDateRangeDispatcher;
use crate::delivery::bot::telegram::dialogues::setup_notifications::TelegramBotSetupNotificationsDispatcher;
use crate::delivery::bot::telegram::dialogues::setup_webhook::TelegramBotSetupWebhookDispatcher;
use crate::delivery::contract::ApplicationDelivery;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

pub struct DeliveryBotMessengerTelegram {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    shared_dependency: Arc<ApplicationSharedDependency>,
}

impl DeliveryBotMessengerTelegram {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
        shared_dependency: Arc<ApplicationSharedDependency>,
    ) -> Self {
        Self {
            executors,
            config,
            shared_dependency,
        }
    }
}

#[async_trait::async_trait]
impl ApplicationDelivery for DeliveryBotMessengerTelegram {
    async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bot = Bot::new(&self.config.telegram.bot_token);

        bot.set_my_commands(commands::builder::TelegramBotCommand::bot_commands())
            .await?;

        let commands_handler = Update::filter_message()
            .filter_command::<commands::builder::TelegramBotCommand>()
            .filter_map(|update: Update| update.from().cloned())
            .endpoint(commands::builder::handle);

        let handler = dptree::entry()
            .enter_dialogue::<Update, InMemStorage<TelegramBotDialogueState>, TelegramBotDialogueState>()
            .branch(commands_handler)
            .branch(
                case![TelegramBotDialogueState::Registration(state)]
                    .branch(TelegramBotDialogueRegistrationDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::ReportByDateRange(state)]
                    .branch(TelegramBotDialogueReportByDateRangeDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::Admin(state)]
                    .branch(TelegramBotDialogueAdminDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::BindRepository(state)]
                    .branch(TelegramBotBindRepositoryDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::SetupWebhook(state)]
                    .branch(TelegramBotSetupWebhookDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::SetupNotifications(state)]
                    .branch(TelegramBotSetupNotificationsDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::Digest(state)]
                    .branch(TelegramBotDigestDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::Notifications(state)]
                    .branch(TelegramBotNotificationsDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::Onboarding(state)]
                    .branch(TelegramBotOnboardingDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::ReleasePlan(state)]
                    .branch(TelegramBotReleasePlanDispatcher::new()),
            )
            .branch(
                case![TelegramBotDialogueState::ReleasePlanSettings(state)]
                    .branch(TelegramBotReleasePlanSettingsDispatcher::new()),
            );

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![
                InMemStorage::<TelegramBotDialogueState>::new(),
                self.executors.clone(),
                self.config.clone(),
                self.shared_dependency.clone()
            ])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
