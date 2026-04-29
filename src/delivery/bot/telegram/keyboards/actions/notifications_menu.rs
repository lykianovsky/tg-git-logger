use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotNotificationsMenuAction {
    #[strum(serialize = "notif_dnd_window")]
    DndWindow,
    #[strum(serialize = "notif_snooze")]
    Snooze,
    #[strum(serialize = "notif_vacation")]
    Vacation,
    #[strum(serialize = "notif_events")]
    Events,
    #[strum(serialize = "notif_priority_only")]
    PriorityOnly,
    #[strum(serialize = "notif_reset")]
    Reset,
    #[strum(serialize = "notif_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotNotificationsMenuAction {
    fn label(&self) -> &'static str {
        match self {
            Self::DndWindow => "🌙 Тихие часы",
            Self::Snooze => "😴 Тишина",
            Self::Vacation => "🏖 Отпуск",
            Self::Events => "🔕 Фильтры событий",
            Self::PriorityOnly => "🚨 Только важное",
            Self::Reset => "🔄 Сбросить к дефолту",
            Self::Cancel => "❌ Закрыть",
        }
    }
}

impl_keyboard_action!(TelegramBotNotificationsMenuAction);
