use crate::domain::version_control::ports::version_control_client::UserPullRequestSummary;

pub struct GetMyPullRequestsResponse {
    pub prs: Vec<UserPullRequestSummary>,
}
