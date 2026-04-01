pub mod admin;
pub mod admin_health_ping;
pub mod admin_health_ping_edit;
pub mod admin_repository;
pub mod admin_repository_delete;
pub mod admin_repository_edit_field;
pub mod admin_task_tracker;
pub mod admin_task_tracker_edit_field;
pub mod admin_user_menu;
pub mod admin_users;
pub mod choose_role;
pub mod confirm;
pub mod date_range;
pub mod digest_list;
pub mod digest_repository;
pub mod digest_type;
pub mod for_who;

pub trait TelegramBotKeyboardAction {
    fn to_callback_data(&self) -> &str;

    fn from_callback_data(data: &str) -> Result<Self, String>
    where
        Self: Sized;

    fn label(&self) -> &'static str;
}
