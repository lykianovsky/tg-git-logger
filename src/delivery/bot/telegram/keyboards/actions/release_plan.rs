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
            Self::Done => "➡️ Дальше",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanReposAction);

pub const REPO_TOGGLE_PREFIX: &str = "rp_repo_";

pub fn repo_toggle_callback(repo_id: i32) -> String {
    format!("{}{}", REPO_TOGGLE_PREFIX, repo_id)
}

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanCallSetupAction {
    #[strum(serialize = "rp_call_default")]
    UseDefault,
    #[strum(serialize = "rp_call_manual")]
    EnterManually,
    #[strum(serialize = "rp_call_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotReleasePlanCallSetupAction {
    fn label(&self) -> &'static str {
        match self {
            Self::UseDefault => "🕐 Использовать дефолт",
            Self::EnterManually => "✏️ Ввести вручную",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanCallSetupAction);

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanOptionalAction {
    #[strum(serialize = "rp_opt_skip")]
    Skip,
    #[strum(serialize = "rp_opt_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotReleasePlanOptionalAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Skip => "⏭ Пропустить",
            Self::Cancel => "❌ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanOptionalAction);
