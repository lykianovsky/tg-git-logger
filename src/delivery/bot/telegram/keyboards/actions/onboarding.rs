use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotOnboardingReposAction {
    #[strum(serialize = "ob_repos_all")]
    SelectAll,
    #[strum(serialize = "ob_repos_done")]
    Done,
    #[strum(serialize = "ob_repos_skip")]
    Skip,
}

impl KeyboardActionLabel for TelegramBotOnboardingReposAction {
    fn label(&self) -> &'static str {
        match self {
            Self::SelectAll => "✅ Выбрать все",
            Self::Done => "✅ Готово",
            Self::Skip => "⏭ Пропустить",
        }
    }
}

impl_keyboard_action!(TelegramBotOnboardingReposAction);

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotOnboardingDndAction {
    #[strum(serialize = "ob_dnd_default")]
    KeepDefault,
    #[strum(serialize = "ob_dnd_custom")]
    Custom,
    #[strum(serialize = "ob_dnd_skip")]
    Skip,
}

impl KeyboardActionLabel for TelegramBotOnboardingDndAction {
    fn label(&self) -> &'static str {
        match self {
            Self::KeepDefault => "🌙 Оставить 20:00–10:00 МСК",
            Self::Custom => "✍️ Свои часы",
            Self::Skip => "⏭ Пропустить",
        }
    }
}

impl_keyboard_action!(TelegramBotOnboardingDndAction);

pub const REPO_TOGGLE_PREFIX: &str = "ob_repo_";

pub fn repo_toggle_callback(repo_id: i32) -> String {
    format!("{}{}", REPO_TOGGLE_PREFIX, repo_id)
}
