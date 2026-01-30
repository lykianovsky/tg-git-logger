pub enum NotificationNotifyError {}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify(&self, telegram_id: i64, text: String) -> Result<(), NotificationNotifyError>;
}