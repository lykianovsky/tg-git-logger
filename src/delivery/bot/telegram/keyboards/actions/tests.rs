use crate::delivery::bot::telegram::keyboards::actions::{
    KeyboardActionLabel, impl_keyboard_action,
};
use strum_macros::{AsRefStr, EnumString};

/// Кнопки карточки прогона. Запуск блока присылает путь блока отдельной кнопкой,
/// поэтому в наборе его нет — только переход к выбору
#[derive(Clone, Debug, PartialEq, Eq, EnumString, AsRefStr)]
pub enum TelegramBotTestsAction {
    #[strum(serialize = "refresh")]
    Refresh,
    #[strum(serialize = "failures")]
    Failures,
    #[strum(serialize = "report")]
    Report,
    #[strum(serialize = "run_all")]
    RunAll,
    #[strum(serialize = "dashboard")]
    Dashboard,
    #[strum(serialize = "readiness")]
    Readiness,
    #[strum(serialize = "rerun_failed")]
    RerunFailed,
    #[strum(serialize = "choose_block")]
    ChooseBlock,
    #[strum(serialize = "cancel_run")]
    CancelRun,
    #[strum(serialize = "connect")]
    Connect,
    #[strum(serialize = "back")]
    Back,
    #[strum(serialize = "close")]
    Close,
}

impl KeyboardActionLabel for TelegramBotTestsAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Refresh => "🔄 Обновить",
            Self::Failures => "❌ Упавшие",
            Self::Report => "📄 Отчёт",
            Self::RunAll => "▶️ Запустить все",
            Self::Dashboard => "📈 Состояние качества",
            Self::Readiness => "🚀 Готов ли к релизу",
            Self::RerunFailed => "🔁 Прогнать упавшие",
            Self::ChooseBlock => "🧩 Запустить блок",
            Self::CancelRun => "🛑 Отменить прогон",
            Self::Connect => "🔌 Подключить тесты",
            Self::Back => "⬅️ Назад",
            Self::Close => "✖️ Закрыть",
        }
    }
}

impl_keyboard_action!(TelegramBotTestsAction);
