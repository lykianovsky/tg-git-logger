pub mod callback;
pub mod callbacks;
mod command;
pub mod context;
mod handlers;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::callbacks::handle_callback;
use crate::delivery::contract::ApplicationDelivery;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

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

        let commands_handler = Update::filter_message()
            .filter_command::<command::TelegramBotCommand>()
            .filter_map(|update: Update| update.from().cloned())
            .endpoint(handlers::handle);

        let callback_handler = Update::filter_callback_query().endpoint(handle_callback);

        let handler = dptree::entry()
            .branch(commands_handler)
            .branch(callback_handler);

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![self.executors.clone()]) // ← Прокидываем executors
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
