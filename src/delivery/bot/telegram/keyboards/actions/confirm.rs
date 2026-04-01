use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotConfirmAction {
    #[strum(serialize = "yes")]
    Yes,
    #[strum(serialize = "no")]
    No,
}

impl KeyboardActionLabel for TelegramBotConfirmAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Yes => "✅ Да",
            Self::No => "❌ Нет",
        }
    }
}

impl_keyboard_action!(TelegramBotConfirmAction);
