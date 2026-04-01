use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotDigestTypeAction {
    #[strum(serialize = "daily")]
    Daily,
    #[strum(serialize = "weekly")]
    Weekly,
    #[strum(serialize = "cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotDigestTypeAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Daily => "📅 Ежедневный",
            Self::Weekly => "📆 Еженедельный",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotDigestTypeAction);
