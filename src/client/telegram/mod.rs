use crate::config::environment::ENV;
use once_cell::sync::Lazy;
use teloxide::Bot;

pub static TELEGRAM_BOT: Lazy<Bot> = Lazy::new(|| {
    return Bot::new(ENV.get("TELEGRAM_BOT_TOKEN"));
});
