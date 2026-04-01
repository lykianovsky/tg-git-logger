use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

pub const REPO_VIEW_SELECT_PREFIX: &str = "repo_view_select_";
pub const REPO_SELECT_PREFIX: &str = "repo_select_";
pub const REPO_DELETE_SELECT_PREFIX: &str = "repo_delete_select_";

pub fn repo_view_select_callback(id: i32) -> String {
    format!("{}{}", REPO_VIEW_SELECT_PREFIX, id)
}

pub fn repo_select_callback(id: i32) -> String {
    format!("{}{}", REPO_SELECT_PREFIX, id)
}

pub fn repo_delete_select_callback(id: i32) -> String {
    format!("{}{}", REPO_DELETE_SELECT_PREFIX, id)
}

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
