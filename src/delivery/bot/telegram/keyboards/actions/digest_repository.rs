use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotDigestRepositoryAction {
    #[strum(serialize = "all")]
    All,
    #[strum(serialize = "cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotDigestRepositoryAction {
    fn label(&self) -> &'static str {
        match self {
            Self::All => "📦 Все репозитории",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotDigestRepositoryAction);
