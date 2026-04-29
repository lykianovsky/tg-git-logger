use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminRepositoryEditField {
    #[strum(serialize = "repo_edit_name")]
    Name,
    #[strum(serialize = "repo_edit_owner")]
    Owner,
    #[strum(serialize = "repo_edit_url")]
    Url,
}

impl KeyboardActionLabel for TelegramBotAdminRepositoryEditField {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminRepositoryEditField::Name => "✏️ Название",
            TelegramBotAdminRepositoryEditField::Owner => "👤 Владелец",
            TelegramBotAdminRepositoryEditField::Url => "🔗 URL",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminRepositoryEditField);
