use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

pub const USER_SELECT_PREFIX: &str = "user_";

pub fn user_select_callback(id: i32) -> String {
    format!("{}{}", USER_SELECT_PREFIX, id)
}

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotAdminUsersListAction {
    #[strum(serialize = "users_select")]
    Select,
    #[strum(serialize = "users_cancel")]
    Cancel,
}

impl TelegramBotKeyboardAction for TelegramBotAdminUsersListAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Select => "👤 Выбрать",
            Self::Cancel => "❌ Отмена",
        }
    }
}
