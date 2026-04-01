use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotAdminRepositoryDeleteAction {
    #[strum(serialize = "repo_delete_yes")]
    Confirm,
    #[strum(serialize = "repo_delete_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotAdminRepositoryDeleteAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Confirm => "🗑 Удалить",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminRepositoryDeleteAction);
