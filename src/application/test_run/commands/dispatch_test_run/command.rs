use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct DispatchTestRunCommand {
    pub repository_id: RepositoryId,
    /// Ветка или commit; пусто — ветка по умолчанию из набора тестов
    pub git_ref: Option<String>,
    /// Аргументы прогона: блок тестов, проект, фильтр. Пусто — все тесты
    pub args: String,
    pub trigger: TestRunTrigger,
    /// Кто запускает: его токеном бот и ходит в CI. Пусто — прогон без инициатора
    pub requested_by_social_user_id: Option<SocialUserId>,
    pub chat_id: Option<SocialChatId>,
}
