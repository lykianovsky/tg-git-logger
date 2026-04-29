use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotNotificationsEventAction {
    #[strum(serialize = "event_pr")]
    Pr,
    #[strum(serialize = "event_review")]
    Review,
    #[strum(serialize = "event_comment")]
    Comment,
    #[strum(serialize = "event_ci")]
    Ci,
    #[strum(serialize = "event_release")]
    Release,
    #[strum(serialize = "event_back")]
    Back,
}

impl KeyboardActionLabel for TelegramBotNotificationsEventAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Pr => "PR",
            Self::Review => "Ревью",
            Self::Comment => "Комментарии",
            Self::Ci => "CI",
            Self::Release => "Релизы",
            Self::Back => "⬅️ Назад",
        }
    }
}

impl_keyboard_action!(TelegramBotNotificationsEventAction);
