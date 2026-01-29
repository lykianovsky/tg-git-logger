use teloxide::prelude::*;

mod commands;

use crate::client::telegram::TELEGRAM_BOT;
use crate::server::telegram_bot::commands::{handle, Command};

pub fn run() {
    tokio::spawn(async move {
        let bot = TELEGRAM_BOT.clone();

        let handler = Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle);

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
}
