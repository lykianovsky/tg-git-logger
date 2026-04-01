use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotForWhoAction {
    #[strum(serialize = "me")]
    Me,
    #[strum(serialize = "repository")]
    Repository,
}

impl KeyboardActionLabel for TelegramBotForWhoAction {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotForWhoAction::Me => "👤 Моя активность",
            TelegramBotForWhoAction::Repository => "📦 Репозиторий",
        }
    }
}

impl_keyboard_action!(TelegramBotForWhoAction);
