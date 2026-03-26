use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminRepositoryEditField {
    #[strum(serialize = "repo_edit_name")]
    Name,
    #[strum(serialize = "repo_edit_owner")]
    Owner,
    #[strum(serialize = "repo_edit_url")]
    Url,
    #[strum(serialize = "repo_edit_external_id")]
    ExternalId,
}

impl TelegramBotKeyboardAction for TelegramBotAdminRepositoryEditField {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminRepositoryEditField::Name => "✏️ Название",
            TelegramBotAdminRepositoryEditField::Owner => "👤 Владелец",
            TelegramBotAdminRepositoryEditField::Url => "🔗 URL",
            TelegramBotAdminRepositoryEditField::ExternalId => "🔢 External ID",
        }
    }
}
