use crate::delivery::bot::telegram::keyboards::actions::{
    impl_keyboard_action, KeyboardActionLabel,
};

use crate::domain::role::value_objects::role_name::RoleName;
use strum_macros::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Debug, Clone)]
pub enum TelegramBotChooseRoleAction {
    #[strum(serialize = "quality_assurance")]
    QualityAssurance,
    #[strum(serialize = "developer")]
    Developer,
}

impl From<TelegramBotChooseRoleAction> for RoleName {
    fn from(value: TelegramBotChooseRoleAction) -> Self {
        match value {
            TelegramBotChooseRoleAction::Developer => RoleName::Developer,
            TelegramBotChooseRoleAction::QualityAssurance => RoleName::QualityAssurance,
        }
    }
}

impl TelegramBotChooseRoleAction {
    pub fn try_from_role(role: &RoleName) -> Option<Self> {
        match role {
            RoleName::Developer => Some(Self::Developer),
            RoleName::QualityAssurance => Some(Self::QualityAssurance),
            _ => None,
        }
    }
}

impl KeyboardActionLabel for TelegramBotChooseRoleAction {
    fn label(&self) -> &'static str {
        match self {
            TelegramBotChooseRoleAction::Developer => "👨‍💻 Разработчик",
            TelegramBotChooseRoleAction::QualityAssurance => "🧪 Тестировщик",
        }
    }
}

impl_keyboard_action!(TelegramBotChooseRoleAction);
