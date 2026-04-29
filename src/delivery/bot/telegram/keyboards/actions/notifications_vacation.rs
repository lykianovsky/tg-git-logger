use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotNotificationsVacationAction {
    #[strum(serialize = "vacation_1d")]
    OneDay,
    #[strum(serialize = "vacation_3d")]
    ThreeDays,
    #[strum(serialize = "vacation_7d")]
    SevenDays,
    #[strum(serialize = "vacation_clear")]
    Clear,
    #[strum(serialize = "vacation_back")]
    Back,
}

impl KeyboardActionLabel for TelegramBotNotificationsVacationAction {
    fn label(&self) -> &'static str {
        match self {
            Self::OneDay => "🗓 1 день",
            Self::ThreeDays => "🗓 3 дня",
            Self::SevenDays => "🗓 7 дней",
            Self::Clear => "↩️ Вернуться из отпуска",
            Self::Back => "⬅️ Назад",
        }
    }
}

impl_keyboard_action!(TelegramBotNotificationsVacationAction);
