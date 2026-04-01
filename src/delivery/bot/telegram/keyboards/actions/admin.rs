use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
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

impl TelegramBotKeyboardAction for TelegramBotAdminAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

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
