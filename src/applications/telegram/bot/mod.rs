mod commands;

use crate::applications::telegram;
use crate::client::telegram::TELEGRAM_BOT;
use std::sync::Arc;
use teloxide::dispatching::Dispatcher;
use teloxide::prelude::*;

pub async fn run() {
    let bot = Arc::clone(&TELEGRAM_BOT);

    let handler = Update::filter_message()
        .filter_command::<telegram::bot::commands::Command>()
        .endpoint(telegram::bot::commands::handle);

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}