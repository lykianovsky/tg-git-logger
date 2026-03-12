use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr)]
pub enum TelegramBotDateRangeAction {
    #[strum(serialize = "last_week")]
    LastWeek,
    #[strum(serialize = "last_2_weeks")]
    Last2Weeks,
    #[strum(serialize = "last_month")]
    LastMonth,
    #[strum(serialize = "this_month")]
    ThisMonth,
}

impl TelegramBotKeyboardAction for TelegramBotDateRangeAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| String::from(e.to_string()))
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotDateRangeAction::LastWeek => "📅 Последняя неделя",
            TelegramBotDateRangeAction::Last2Weeks => "📅 Последние 2 недели",
            TelegramBotDateRangeAction::LastMonth => "📅 Последний месяц",
            TelegramBotDateRangeAction::ThisMonth => "📅 Этот месяц",
        }
    }
}
