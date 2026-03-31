use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminRepositoryAction {
    #[strum(serialize = "admin_repository_view")]
    View,
    #[strum(serialize = "admin_repository_create")]
    Create,
    #[strum(serialize = "admin_repository_edit")]
    Edit,
    #[strum(serialize = "admin_repository_delete")]
    Delete,
}

impl TelegramBotKeyboardAction for TelegramBotAdminRepositoryAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminRepositoryAction::View => "🔍 Просмотр",
            TelegramBotAdminRepositoryAction::Create => "➕ Создать новый",
            TelegramBotAdminRepositoryAction::Edit => "✏️ Редактировать",
            TelegramBotAdminRepositoryAction::Delete => "🗑 Удалить",
        }
    }
}
