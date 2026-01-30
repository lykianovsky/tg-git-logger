mod commands;
mod bind;
mod context;
mod start;
mod login;

use crate::applications::state::ApplicationState;
use crate::applications::telegram;
use crate::applications::telegram::bot::commands::Command;
use crate::client::telegram::TELEGRAM_BOT;
use std::sync::Arc;
use teloxide::dispatching::Dispatcher;
use teloxide::prelude::*;
use teloxide::types::User;

pub async fn run(application_state: Arc<ApplicationState>) {
    let bot = Arc::clone(&TELEGRAM_BOT);

    let handler = Update::filter_message()
        .filter_command::<telegram::bot::commands::Command>()
        .filter_map(|update: Update| update.from().cloned())
        .endpoint({
            move |bot: Arc<Bot>, msg: Message, user: User, cmd: Command| {
                let application_state = Arc::clone(&application_state);
                telegram::bot::commands::handle(bot, user, msg, cmd, application_state)
            }
        });

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}