use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotAdminHealthPingAction {
    #[strum(serialize = "hp_create")]
    Create,
    #[strum(serialize = "hp_edit")]
    Edit,
    #[strum(serialize = "hp_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotAdminHealthPingAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Create => "➕ Создать",
            Self::Edit => "✏️ Редактировать",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminHealthPingAction);
