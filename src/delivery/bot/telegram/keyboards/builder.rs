use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct KeyboardBuilder {
    buttons: Vec<Vec<InlineKeyboardButton>>,
}

impl KeyboardBuilder {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }

    pub fn row<Action: TelegramBotKeyboardAction>(mut self, actions: Vec<Action>) -> Self {
        let buttons = actions
            .into_iter()
            .map(|a| InlineKeyboardButton::callback(a.label(), a.to_callback_data()))
            .collect();

        self.buttons.push(buttons);
        self
    }

    pub fn build(self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(self.buttons)
    }
}
