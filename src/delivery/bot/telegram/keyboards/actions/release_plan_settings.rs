use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

/// Prefix for callback "view release plan card" — list item callback.
pub const RPS_VIEW_PREFIX: &str = "rps_view_";
/// Prefix for callback "open settings menu for plan id" — opens edit menu (Admin/PM only).
pub const RPS_EDIT_PREFIX: &str = "rps_edit_";
/// Callback "back to releases list" — re-renders the plan list message.
pub const RPS_BACK_TO_LIST: &str = "rps_back_to_list";
/// Prefix for callback "cancel plan" — opens reason input (Admin/PM only).
pub const RPS_CANCEL_BTN_PREFIX: &str = "rps_cancel_";
/// Prefix for callback "complete plan" — opens confirmation (Admin/PM only).
pub const RPS_COMPLETE_BTN_PREFIX: &str = "rps_complete_";
/// Prefix for repo toggle inside settings menu.
pub const RPS_REPO_TOGGLE_PREFIX: &str = "rps_repo_";

pub fn rps_view_callback(plan_id: i32) -> String {
    format!("{}{}", RPS_VIEW_PREFIX, plan_id)
}
pub fn rps_edit_callback(plan_id: i32) -> String {
    format!("{}{}", RPS_EDIT_PREFIX, plan_id)
}
pub fn rps_cancel_btn_callback(plan_id: i32) -> String {
    format!("{}{}", RPS_CANCEL_BTN_PREFIX, plan_id)
}
pub fn rps_complete_btn_callback(plan_id: i32) -> String {
    format!("{}{}", RPS_COMPLETE_BTN_PREFIX, plan_id)
}
pub fn rps_repo_toggle_callback(repo_id: i32) -> String {
    format!("{}{}", RPS_REPO_TOGGLE_PREFIX, repo_id)
}

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanSettingsMenuAction {
    #[strum(serialize = "rps_edit_planned_date")]
    EditPlannedDate,
    #[strum(serialize = "rps_edit_call_date")]
    EditCallDate,
    #[strum(serialize = "rps_remove_call")]
    RemoveCall,
    #[strum(serialize = "rps_edit_meeting")]
    EditMeeting,
    #[strum(serialize = "rps_remove_meeting")]
    RemoveMeeting,
    #[strum(serialize = "rps_edit_note")]
    EditNote,
    #[strum(serialize = "rps_remove_note")]
    RemoveNote,
    #[strum(serialize = "rps_edit_repos")]
    EditRepos,
    #[strum(serialize = "rps_close")]
    Close,
}

impl KeyboardActionLabel for TelegramBotReleasePlanSettingsMenuAction {
    fn label(&self) -> &'static str {
        match self {
            Self::EditPlannedDate => "📅 Дата релиза",
            Self::EditCallDate => "🕐 Дата/время созвона",
            Self::RemoveCall => "🚫 Удалить созвон",
            Self::EditMeeting => "🔗 Ссылка на встречу",
            Self::RemoveMeeting => "🚫 Удалить ссылку",
            Self::EditNote => "📝 Заметка",
            Self::RemoveNote => "🚫 Удалить заметку",
            Self::EditRepos => "📦 Репозитории",
            Self::Close => "❌ Закрыть",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanSettingsMenuAction);

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanSettingsConfirmAction {
    #[strum(serialize = "rps_confirm_yes")]
    Yes,
    #[strum(serialize = "rps_confirm_no")]
    No,
}

impl KeyboardActionLabel for TelegramBotReleasePlanSettingsConfirmAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Yes => "✅ Да",
            Self::No => "↩️ Нет",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanSettingsConfirmAction);

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotReleasePlanSettingsReposAction {
    #[strum(serialize = "rps_repos_save")]
    Save,
    #[strum(serialize = "rps_repos_cancel")]
    Cancel,
}

impl KeyboardActionLabel for TelegramBotReleasePlanSettingsReposAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Save => "💾 Сохранить",
            Self::Cancel => "↩️ Отмена",
        }
    }
}

impl_keyboard_action!(TelegramBotReleasePlanSettingsReposAction);
