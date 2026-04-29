use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanReposAction {
    #[strum(serialize = "rp_all")]
    SelectAll,
    #[strum(serialize = "rp_done")]
    Done,
    #[strum(serialize = "rp_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotReleasePlanReposAction {
    fn label(&self) -> &'static str {
        match self {
            Self::SelectAll => "✅ Выбрать все",
            Self::Done => "✅ Создать",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanReposAction);

pub const REPO_TOGGLE_PREFIX: &str = "rp_repo_";

pub fn repo_toggle_callback(repo_id: i32) -> String {
    format!("{}{}", REPO_TOGGLE_PREFIX, repo_id)
}
