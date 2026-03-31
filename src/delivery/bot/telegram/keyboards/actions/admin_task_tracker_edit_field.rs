use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

/// Поля, доступные для ручного редактирования.
/// SpaceId/QaColumnId/PathToCard теперь выбираются через API, не вручную.
#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminTaskTrackerEditField {
    #[strum(serialize = "admin_tt_edit_extract_pattern")]
    ExtractPattern,
    #[strum(serialize = "admin_tt_edit_reconfigure")]
    Reconfigure,
}

impl TelegramBotKeyboardAction for TelegramBotAdminTaskTrackerEditField {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminTaskTrackerEditField::ExtractPattern => "✏️ Regex паттерн",
            TelegramBotAdminTaskTrackerEditField::Reconfigure => "🔄 Перевыбрать колонку/спейс",
        }
    }
}
