mod commands;
pub mod context;
pub mod dialogues;
mod keyboards;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::report::{
    TelegramBotReportByDateRangeDialogue, TelegramBotReportByDateRangeDialogueState,
};
use crate::delivery::contract::ApplicationDelivery;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
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

        bot.set_my_commands(commands::builder::TelegramBotCommand::bot_commands())
            .await?;

        let commands_handler = Update::filter_message()
            .filter_command::<commands::builder::TelegramBotCommand>()
            .filter_map(|update: Update| update.from().cloned())
            .endpoint(commands::builder::handle);

        let dialog_message_handler = Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<TelegramBotReportByDateRangeDialogueState>, TelegramBotReportByDateRangeDialogueState>();

        let callback_handler = Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, InMemStorage<TelegramBotReportByDateRangeDialogueState>, TelegramBotReportByDateRangeDialogueState>()
            .branch(
                case![TelegramBotReportByDateRangeDialogueState::For]
                    .endpoint(TelegramBotReportByDateRangeDialogue::choose_for_who)
            )
            .branch(
                case![TelegramBotReportByDateRangeDialogueState::DateRange{for_who_action}]
                    .endpoint(TelegramBotReportByDateRangeDialogue::create_report_by_date_range)
            );

        let handler = dptree::entry()
            .enter_dialogue::<Update, InMemStorage<TelegramBotReportByDateRangeDialogueState>, TelegramBotReportByDateRangeDialogueState>()
            .branch(commands_handler)
            .branch(callback_handler)
            .branch(dialog_message_handler);

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![
                InMemStorage::<TelegramBotReportByDateRangeDialogueState>::new(),
                self.executors.clone(),
                self.config.clone()
            ])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
