use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotNotificationsSnoozeAction {
    #[strum(serialize = "snooze_2h")]
    TwoHours,
    #[strum(serialize = "snooze_4h")]
    FourHours,
    #[strum(serialize = "snooze_morning")]
    UntilMorning,
    #[strum(serialize = "snooze_clear")]
    Clear,
    #[strum(serialize = "snooze_back")]
    Back,
}

impl KeyboardActionLabel for TelegramBotNotificationsSnoozeAction {
    fn label(&self) -> &'static str {
        match self {
            Self::TwoHours => "⏱ 2 часа",
            Self::FourHours => "⏱ 4 часа",
            Self::UntilMorning => "🌅 До утра",
            Self::Clear => "🔕 Снять snooze",
            Self::Back => "⬅️ Назад",
        }
    }
}

impl_keyboard_action!(TelegramBotNotificationsSnoozeAction);
