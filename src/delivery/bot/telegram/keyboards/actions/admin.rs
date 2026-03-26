use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminAction {
    #[strum(serialize = "admin_configure_repository")]
    ConfigureRepository,
    #[strum(serialize = "admin_configure_task_tracker")]
    ConfigureTaskTracker,
}

impl TelegramBotKeyboardAction for TelegramBotAdminAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminAction::ConfigureRepository => "⚙️ Настроить репозиторий",
            TelegramBotAdminAction::ConfigureTaskTracker => "⚙️ Настроить таск-трекер",
        }
    }
}
