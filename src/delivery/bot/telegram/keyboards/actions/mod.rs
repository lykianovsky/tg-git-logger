pub mod date_range;
pub mod for_who;

pub trait TelegramBotKeyboardAction {
    fn to_callback_data(&self) -> &'static str;

    fn from_callback_data(data: &str) -> Result<Self, String>
    where
        Self: Sized;

    fn label(&self) -> &'static str;
}
