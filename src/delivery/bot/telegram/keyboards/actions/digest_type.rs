use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
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

impl TelegramBotKeyboardAction for TelegramBotDigestTypeAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Daily => "📅 Ежедневный",
            Self::Weekly => "📆 Еженедельный",
            Self::Cancel => "❌ Отмена",
        }
    }
}
