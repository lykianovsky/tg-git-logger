use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminTaskTrackerAction {
    #[strum(serialize = "admin_tt_view")]
    View,
    #[strum(serialize = "admin_tt_edit")]
    Edit,
    #[strum(serialize = "admin_tt_reconfigure")]
    Reconfigure,
}

impl KeyboardActionLabel for TelegramBotAdminTaskTrackerAction {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminTaskTrackerAction::View => "👁 Посмотреть настройки",
            TelegramBotAdminTaskTrackerAction::Edit => "✏️ Редактировать поле",
            TelegramBotAdminTaskTrackerAction::Reconfigure => "🔄 Настроить заново",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminTaskTrackerAction);
