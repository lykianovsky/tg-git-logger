use crate::delivery::bot::telegram::dialogues::release_plan::TelegramBotReleasePlanState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::Message;

pub struct TelegramBotReleasePlanCommandHandler {
    bot: Bot,
    msg: Message,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotReleasePlanCommandHandler {
    pub fn new(bot: Bot, msg: Message, dialogue: Arc<TelegramBotDialogueType>) -> Self {
        Self { bot, msg, dialogue }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.dialogue
            .update(TelegramBotDialogueState::ReleasePlan(
                TelegramBotReleasePlanState::EnterDate,
            ))
            .await?;

        self.bot
            .send_message(
                self.msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.enter_date").to_string(),
            )
            .await?;
        Ok(())
    }
}
