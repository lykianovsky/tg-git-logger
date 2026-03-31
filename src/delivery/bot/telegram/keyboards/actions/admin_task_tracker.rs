use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminTaskTrackerAction {
    #[strum(serialize = "admin_tt_view")]
    View,
    #[strum(serialize = "admin_tt_edit")]
    Edit,
    #[strum(serialize = "admin_tt_reconfigure")]
    Reconfigure,
}

impl TelegramBotKeyboardAction for TelegramBotAdminTaskTrackerAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminTaskTrackerAction::View => "👁 Посмотреть настройки",
            TelegramBotAdminTaskTrackerAction::Edit => "✏️ Редактировать поле",
            TelegramBotAdminTaskTrackerAction::Reconfigure => "🔄 Настроить заново",
        }
    }
}
