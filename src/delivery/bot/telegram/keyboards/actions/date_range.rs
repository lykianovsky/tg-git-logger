use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;

pub enum TelegramBotDateRangeAction {
    LastWeek,
    Last2Weeks,
    LastMonth,
    ThisMonth,
}

impl TelegramBotKeyboardAction for TelegramBotDateRangeAction {
    fn to_callback_data(&self) -> &'static str {
        match self {
            TelegramBotDateRangeAction::LastWeek => "date:last_week",
            TelegramBotDateRangeAction::Last2Weeks => "date:last_2_weeks",
            TelegramBotDateRangeAction::LastMonth => "date:last_month",
            TelegramBotDateRangeAction::ThisMonth => "date:this_month",
        }
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        match data {
            "date:last_week" => Ok(Self::LastWeek),
            "date:last_2_weeks" => Ok(Self::Last2Weeks),
            "date:last_month" => Ok(Self::LastMonth),
            "date:this_month" => Ok(Self::ThisMonth),
            _ => Err(format!("Unknown action type: {data}")),
        }
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
