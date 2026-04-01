use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
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

impl KeyboardActionLabel for TelegramBotAdminUsersListAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Select => "👤 Выбрать",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminUsersListAction);
