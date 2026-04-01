use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};
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

impl KeyboardActionLabel for TelegramBotAdminHealthPingEditAction {
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

impl_keyboard_action!(TelegramBotAdminHealthPingEditAction);
