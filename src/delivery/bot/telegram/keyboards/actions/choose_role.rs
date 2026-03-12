use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;

use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotChooseRoleAction {
    #[strum(serialize = "quality_assurance")]
    QualityAssurance,
    #[strum(serialize = "developer")]
    Developer,
}

impl TelegramBotKeyboardAction for TelegramBotChooseRoleAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| String::from(e.to_string()))
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotChooseRoleAction::Developer => "👨‍💻 Разработчик",
            TelegramBotChooseRoleAction::QualityAssurance => "🧪 Тестировщик",
        }
    }
}
