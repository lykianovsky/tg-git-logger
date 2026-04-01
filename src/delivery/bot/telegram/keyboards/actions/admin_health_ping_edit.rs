use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

#[derive(Clone, Debug, EnumString, AsRefStr)]
pub enum TelegramBotAdminHealthPingEditAction {
    #[strum(serialize = "hp_edit_name")]
    Name,
    #[strum(serialize = "hp_edit_url")]
    Url,
    #[strum(serialize = "hp_edit_interval")]
    Interval,
    #[strum(serialize = "hp_toggle")]
    Toggle,
    #[strum(serialize = "hp_delete")]
    Delete,
}

impl TelegramBotKeyboardAction for TelegramBotAdminHealthPingEditAction {
    fn to_callback_data(&self) -> &str {
        self.as_ref()
    }

    fn from_callback_data(data: &str) -> Result<Self, String> {
        Self::from_str(data).map_err(|e| e.to_string())
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Name => "📝 Имя",
            Self::Url => "🔗 URL",
            Self::Interval => "⏱ Интервал",
            Self::Toggle => "🔄 Вкл/Выкл",
            Self::Delete => "🗑 Удалить",
        }
    }
}
