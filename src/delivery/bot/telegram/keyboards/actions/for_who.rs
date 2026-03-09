use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;

#[derive(Clone, Debug)]
pub enum TelegramBotForWhoAction {
    Me,
    Repository,
}

impl TelegramBotKeyboardAction for TelegramBotForWhoAction {
    fn to_callback_data(&self) -> &'static str {
        match self {
            TelegramBotForWhoAction::Me => "who:me",
            TelegramBotForWhoAction::Repository => "who:repository",
        }
    }

    fn from_callback_data(data: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match data {
            "who:me" => Ok(Self::Me),
            "who:repository" => Ok(Self::Repository),
            _ => Err(format!("Unknown action type: {data}")),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            TelegramBotForWhoAction::Me => "👤 Моя активность",
            TelegramBotForWhoAction::Repository => "📦 Репозиторий",
        }
    }
}
