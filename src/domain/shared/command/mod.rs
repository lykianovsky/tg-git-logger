pub trait CommandExecutor {
    type Command;
    type Response;
    type Error;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error>;
}
