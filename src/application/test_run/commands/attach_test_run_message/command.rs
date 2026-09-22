use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;

/// Сообщение-карточка прогона: по нему бот обновляет статус, пока прогон идёт
pub struct AttachTestRunMessageCommand {
    pub test_run_id: TestRunId,
    pub chat_id: SocialChatId,
    pub message_id: i32,
}
