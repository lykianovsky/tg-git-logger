use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotDigestListAction {
    #[strum(serialize = "digest_create")]
    Create,
    #[strum(serialize = "cancel")]
    Cancel,
}

impl TelegramBotDigestListAction {
    pub const TOGGLE_PREFIX: &'static str = "toggle_";
    pub const DELETE_PREFIX: &'static str = "delete_";

    pub fn toggle_callback(id: i32) -> String {
        format!("{}{}", Self::TOGGLE_PREFIX, id)
    }

    pub fn delete_callback(id: i32) -> String {
        format!("{}{}", Self::DELETE_PREFIX, id)
    }
}

impl KeyboardActionLabel for TelegramBotDigestListAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Create => "➕ Создать",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotDigestListAction);
