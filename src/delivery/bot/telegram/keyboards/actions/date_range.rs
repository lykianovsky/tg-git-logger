use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotDateRangeAction {
    #[strum(serialize = "last_week")]
    LastWeek,
    #[strum(serialize = "last_2_weeks")]
    Last2Weeks,
    #[strum(serialize = "last_month")]
    LastMonth,
    #[strum(serialize = "this_month")]
    ThisMonth,
    #[strum(serialize = "custom_date_range")]
    Custom,
}

impl KeyboardActionLabel for TelegramBotDateRangeAction {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotDateRangeAction::LastWeek => "📅 Последняя неделя",
            TelegramBotDateRangeAction::Last2Weeks => "📅 Последние 2 недели",
            TelegramBotDateRangeAction::LastMonth => "📅 Последний месяц",
            TelegramBotDateRangeAction::ThisMonth => "📅 Этот месяц",
            TelegramBotDateRangeAction::Custom => "📆 Свой диапазон",
        }
    }
}

impl_keyboard_action!(TelegramBotDateRangeAction);
