use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum TelegramBotCommand {
    #[command(description = "Запустить бота")]
    Start,
    #[command(description = "Создать пользователя")]
    Register,
    #[command(description = "Получить отчет")]
    WeeklyReport,
}
