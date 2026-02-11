mod command;
pub mod context;
mod handlers;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

pub struct DeliveryBotMessengerTelegram {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryBotMessengerTelegram {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        Self { executors, config }
    }
}

#[async_trait::async_trait]
impl ApplicationDelivery for DeliveryBotMessengerTelegram {
    async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bot = Bot::new(&self.config.telegram.bot_token);

        bot.set_my_commands(command::TelegramBotCommand::bot_commands())
            .await?;

        let handler = Update::filter_message()
            .filter_command::<command::TelegramBotCommand>()
            .filter_map(|update: Update| update.from().cloned())
            .endpoint(handlers::handle);

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![self.executors.clone()]) // ← Прокидываем executors
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
