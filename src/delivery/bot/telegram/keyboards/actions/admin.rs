use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotAdminAction {
    #[strum(serialize = "admin_configure_repository")]
    ConfigureRepository,
    #[strum(serialize = "admin_configure_task_tracker")]
    ConfigureTaskTracker,
    #[strum(serialize = "admin_queues_stats")]
    QueuesStats,

    #[strum(serialize = "admin_health_pings")]
    HealthPings,

    #[strum(serialize = "admin_manage_users")]
    ManageUsers,
}

impl KeyboardActionLabel for TelegramBotAdminAction {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotAdminAction::ConfigureRepository => "📦 Репозитории",
            TelegramBotAdminAction::ConfigureTaskTracker => "⚙️ Настроить таск-трекер",
            TelegramBotAdminAction::QueuesStats => "📊 Очереди и воркеры",
            TelegramBotAdminAction::HealthPings => "🏓 Пинги",
            TelegramBotAdminAction::ManageUsers => "👥 Пользователи",
        }
    }
}

impl_keyboard_action!(TelegramBotAdminAction);
