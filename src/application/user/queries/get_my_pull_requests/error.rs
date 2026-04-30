use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetMyPullRequestsError {
    #[error("User not found")]
    UserNotFound,
    #[error("GitHub account not linked")]
    NoGithubAccount,
    #[error("Database error: {0}")]
    DbError(String),
    #[error("GitHub API error: {0}")]
    GithubError(String),
}
