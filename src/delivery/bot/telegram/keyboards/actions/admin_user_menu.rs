use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotAdminUserMenuAction {
    #[strum(serialize = "um_toggle")]
    Toggle,
    #[strum(serialize = "um_assign_role")]
    AssignRole,
    #[strum(serialize = "um_remove_role")]
    RemoveRole,
}

impl TelegramBotKeyboardAction for TelegramBotAdminUserMenuAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Toggle => "🔄 Вкл/Выкл",
            Self::AssignRole => "➕ Назначить роль",
            Self::RemoveRole => "➖ Убрать роль",
        }
    }
}
