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
pub mod notifications_events;
pub mod notifications_menu;
pub mod notifications_snooze;
pub mod notifications_vacation;
pub mod onboarding;
pub mod release_plan;

pub trait TelegramBotKeyboardAction {
    fn to_callback_data(&self) -> &str;

    fn from_callback_data(data: &str) -> Result<Self, String>
    where
        Self: Sized;

    fn label(&self) -> &'static str;
}

/// Implements `to_callback_data` and `from_callback_data` for enums
/// that derive `AsRefStr` + `EnumString` from strum.
///
/// Usage:
/// ```ignore
/// impl_keyboard_action!(TelegramBotConfirmAction);
/// ```
///
/// After this macro, only `label()` needs a manual impl.
macro_rules! impl_keyboard_action {
    ($ty:ty) => {
        impl $crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction
            for $ty
        {
            fn to_callback_data(&self) -> &str {
                self.as_ref()
            }

            fn from_callback_data(data: &str) -> Result<Self, String> {
                <Self as std::str::FromStr>::from_str(data)
                    .map_err(|e| e.to_string())
            }

            fn label(&self) -> &'static str {
                <Self as $crate::delivery::bot::telegram::keyboards::actions::KeyboardActionLabel>::label(self)
            }
        }
    };
}

/// Trait for defining labels on keyboard actions, used by
/// [`impl_keyboard_action!`].
pub trait KeyboardActionLabel {
    fn label(&self) -> &'static str;
}

pub(crate) use impl_keyboard_action;
