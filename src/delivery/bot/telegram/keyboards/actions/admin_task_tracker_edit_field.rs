use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

/// Поля, доступные для ручного редактирования.
/// SpaceId/QaColumnId/PathToCard теперь выбираются через API, не вручную.
#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminTaskTrackerEditField {
    #[strum(serialize = "admin_tt_edit_extract_pattern")]
    ExtractPattern,
    #[strum(serialize = "admin_tt_edit_reconfigure")]
    Reconfigure,
}

impl KeyboardActionLabel for TelegramBotAdminTaskTrackerEditField {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminTaskTrackerEditField::ExtractPattern => "✏️ Regex паттерн",
            TelegramBotAdminTaskTrackerEditField::Reconfigure => "🔄 Перевыбрать колонку/спейс",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminTaskTrackerEditField);
