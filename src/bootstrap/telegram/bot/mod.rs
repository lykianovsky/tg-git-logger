use crate::app::telegram;
use crate::infrastructure::delivery::state::ApplicationState;
use crate::infrastructure::integrations::telegram::TELEGRAM_BOT;
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
            move |bot: Arc<Bot>, msg: Message, user: User, cmd: telegram::bot::commands::Command| {
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
