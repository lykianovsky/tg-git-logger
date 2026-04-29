use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
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

impl KeyboardActionLabel for TelegramBotAdminUserMenuAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Toggle => "🔄 Вкл/Выкл",
            Self::AssignRole => "➕ Назначить роль",
            Self::RemoveRole => "➖ Убрать роль",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminUserMenuAction);
