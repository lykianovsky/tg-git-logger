use crate::config::environment::ENV;
use once_cell::sync::Lazy;
use std::sync::Arc;
use teloxide::Bot;

pub static TELEGRAM_BOT: Lazy<Arc<Bot>> = Lazy::new(|| {
    return Arc::new(Bot::new(ENV.get("TELEGRAM_BOT_TOKEN")));
});
