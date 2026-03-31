use crate::domain::shared::date::range::DateRange;
use crate::domain::version_control::ports::version_control_client::{
    VersionControlClient, VersionControlClientBranchCheckError,
    VersionControlClientDateRangeReportError, VersionControlClientGetUserError,
    VersionControlClientGetUserResponse,
};
use crate::domain::version_control::value_objects::report::{
    VersionControlDateRangeReport, VersionControlDateRangeReportAuthor,
    VersionControlDateRangeReportCommit, VersionControlDateRangeReportPullRequest,
};
use crate::infrastructure::integrations::version_control::github::error::{
    GithubGraphQLError, GithubGraphQLErrorType, GithubGraphQLResponse,
};
use async_trait::async_trait;
use chrono::Utc;
use graphql_client::GraphQLQuery;
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubClientError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse response JSON: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Unexpected status code: {0}")]
    Status(reqwest::StatusCode),

    #[error("GraphQL error")]
    GraphQL(Vec<GithubGraphQLError>),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

impl GithubClientError {
    pub fn is_not_found(&self) -> bool {
        self.has_error_type(GithubGraphQLErrorType::NotFound)
    }

    pub fn is_forbidden(&self) -> bool {
        self.has_error_type(GithubGraphQLErrorType::Forbidden)
    }

    pub fn is_rate_limited(&self) -> bool {
        self.has_error_type(GithubGraphQLErrorType::RateLimited)
    }

    pub fn get_error_by_type(
        &self,
        error_type: GithubGraphQLErrorType,
    ) -> Option<&GithubGraphQLError> {
        match self {
            GithubClientError::GraphQL(errors) => {
                errors.iter().find(|e| e.error_type == error_type)
            }
            _ => None,
        }
    }

    pub fn has_error_type(&self, error_type: GithubGraphQLErrorType) -> bool {
        matches!(self, GithubClientError::GraphQL(errors)
            if errors.iter().any(|e| {
                tracing::debug!("Graphql Error Type {}", e.error_type);
                e.error_type == error_type
            })
        )
    }
}

type GitObjectID = String;
type GitTimestamp = chrono::DateTime<Utc>;
type BigInt = String;
type DateTime = chrono::DateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/infrastructure/integrations/version_control/github/graphql/schema.docs.graphql",
    query_path = "src/infrastructure/integrations/version_control/github/graphql/queries/get_user.graphql"
)]
pub struct GithubUser;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/infrastructure/integrations/version_control/github/graphql/schema.docs.graphql",
    query_path = "src/infrastructure/integrations/version_control/github/graphql/queries/get_commit_history.graphql"
)]
pub struct GithubCommitHistory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/infrastructure/integrations/version_control/github/graphql/schema.docs.graphql",
    query_path = "src/infrastructure/integrations/version_control/github/graphql/queries/get_pull_requests.graphql"
)]
pub struct GithubPullRequests;

pub struct GithubVersionControlClient {
    base: String,
    client: Client,
}

impl GithubVersionControlClient {
    pub fn new(base: String) -> Self {
        Self {
            base,
            client: Client::new(),
        }
    }

    async fn graphql<Q: GraphQLQuery>(
        &self,
        access_token: &str,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData, GithubClientError> {
        let body = Q::build_query(variables);

        let resp = self
            .client
            .post(format!("{}/graphql", self.base))
            .bearer_auth(access_token)
            .header("User-Agent", "Telegram-Git-App")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            return Err(GithubClientError::Status(status));
        }

        let text = resp.text().await?;
        tracing::debug!("GitHub GraphQL response: {}", text);

        let result: GithubGraphQLResponse<Q::ResponseData> = serde_json::from_str(&text)?;

        if let Some(errors) = result.errors {
            return Err(GithubClientError::GraphQL(errors));
        }

        result
            .data
            .ok_or_else(|| GithubClientError::InvalidResponse("No data in response".to_string()))
    }
}

#[async_trait]
impl VersionControlClient for GithubVersionControlClient {
    async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VersionControlClientGetUserResponse, VersionControlClientGetUserError> {
        let data = self
            .graphql::<GithubUser>(access_token, github_user::Variables {})
            .await
            .map_err(|e| {
                if let Some(graphql_error) = e.get_error_by_type(GithubGraphQLErrorType::Forbidden)
                {
                    return VersionControlClientGetUserError::Unauthorized(
                        graphql_error.message.to_string(),
                    );
                }

                VersionControlClientGetUserError::Transport(e.to_string())
            })?;

        Ok(VersionControlClientGetUserResponse {
            id: data.viewer.id.unwrap_or_default(),
            login: data.viewer.login,
            email: Some(data.viewer.email),
        })
    }

    async fn get_details_by_range(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        branch: &str,
        date_range: &DateRange,
        author: Option<&str>,
    ) -> Result<VersionControlDateRangeReport, VersionControlClientDateRangeReportError> {
        // ── Fetch all commits with cursor pagination ───────────────────────────
        let mut commits = Vec::new();
        let mut commit_cursor: Option<String> = None;
        let mut branch_exists = true;

        loop {
            let data = self
                .graphql::<GithubCommitHistory>(
                    access_token,
                    github_commit_history::Variables {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        branch: branch.to_string(),
                        since: date_range.since,
                        until: date_range.until,
                        after: commit_cursor.clone(),
                    },
                )
                .await
                .map_err(|e| {
                    if let Some(err) = e.get_error_by_type(GithubGraphQLErrorType::Forbidden) {
                        return VersionControlClientDateRangeReportError::Unauthorized(
                            err.message.to_string(),
                        );
                    }
                    VersionControlClientDateRangeReportError::Transport(e.to_string())
                })?;

            let Some(repository) = data.repository else {
                break;
            };

            let Some(ref_) = repository.ref_ else {
                branch_exists = false;
                break;
            };

            let Some(target) = ref_.target else {
                break;
            };

            let github_commit_history::GithubCommitHistoryRepositoryRefTarget::Commit(
                commit_target,
            ) = target
            else {
                break;
            };

            let history = commit_target.history;
            let has_next = history.page_info.has_next_page;
            let end_cursor = history.page_info.end_cursor;

            if let Some(nodes) = history.nodes {
                for commit in nodes.into_iter().flatten() {
                    commits.push(VersionControlDateRangeReportCommit {
                        sha: commit.oid,
                        message: commit.message,
                        authored_at: commit.committed_date,
                        additions: commit.additions,
                        deletions: commit.deletions,
                        changed_files: commit.changed_files_if_available,
                        author: commit.author.map(|a| VersionControlDateRangeReportAuthor {
                            login: a.user.and_then(|u| u.login.into()),
                            name: a.name,
                            email: a.email,
                        }),
                    });
                }
            }

            if !has_next {
                break;
            }
            commit_cursor = end_cursor;
        }

        // If the branch doesn't exist, report it early
        if !branch_exists {
            return Err(VersionControlClientDateRangeReportError::BranchNotFound(
                branch.to_string(),
            ));
        }

        // ── Fetch all PRs with cursor pagination ───────────────────────────────
        let author_filter = author.map(|a| format!(" author:{}", a)).unwrap_or_default();
        let pr_search = format!(
            "repo:{}/{} is:pr created:{}..{}{}",
            owner,
            repo,
            date_range.since.format("%Y-%m-%d"),
            date_range.until.format("%Y-%m-%d"),
            author_filter,
        );

        let mut pull_requests = Vec::new();
        let mut pr_cursor: Option<String> = None;

        loop {
            let data = self
                .graphql::<GithubPullRequests>(
                    access_token,
                    github_pull_requests::Variables {
                        pr_search: pr_search.clone(),
                        after: pr_cursor.clone(),
                    },
                )
                .await
                .map_err(|e| {
                    if let Some(err) = e.get_error_by_type(GithubGraphQLErrorType::Forbidden) {
                        return VersionControlClientDateRangeReportError::Unauthorized(
                            err.message.to_string(),
                        );
                    }
                    VersionControlClientDateRangeReportError::Transport(e.to_string())
                })?;

            let has_next = data.pull_requests.page_info.has_next_page;
            let end_cursor = data.pull_requests.page_info.end_cursor;

            if let Some(nodes) = data.pull_requests.nodes {
                for node in nodes.into_iter().flatten() {
                    if let github_pull_requests::GithubPullRequestsPullRequestsNodes::PullRequest(
                        pr,
                    ) = node
                    {
                        pull_requests.push(VersionControlDateRangeReportPullRequest {
                            number: pr.number,
                            title: pr.title,
                            state: serde_json::to_value(&pr.state)
                                .ok()
                                .and_then(|v| v.as_str().map(|s| s.to_lowercase()))
                                .unwrap_or_else(|| "unknown".to_string()),
                            created_at: pr.created_at,
                            merged_at: pr.merged_at,
                            closed_at: pr.closed_at,
                            additions: pr.additions,
                            deletions: pr.deletions,
                            changed_files: pr.changed_files,
                            author: pr.author.map(|a| a.login),
                        });
                    }
                }
            }

            if !has_next {
                break;
            }
            pr_cursor = end_cursor;
        }

        // When a specific author is requested, filter commits to that login.
        if let Some(login) = author {
            commits.retain(|c| {
                c.author
                    .as_ref()
                    .and_then(|a| a.login.as_deref())
                    .map(|l| l.eq_ignore_ascii_case(login))
                    .unwrap_or(false)
            });
        }

        tracing::debug!(
            repo = %repo,
            commits = commits.len(),
            prs = pull_requests.len(),
            "GitHub report fetched (paginated)"
        );

        Ok(VersionControlDateRangeReport {
            commits,
            pull_requests,
        })
    }

    async fn branch_exists(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<bool, VersionControlClientBranchCheckError> {
        let url = format!("{}/repos/{}/{}/branches/{}", self.base, owner, repo, branch);

        let resp = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("User-Agent", "Telegram-Git-App")
            .send()
            .await
            .map_err(|e| VersionControlClientBranchCheckError::Transport(e.to_string()))?;

        match resp.status() {
            status if status.is_success() => Ok(true),
            status if status == reqwest::StatusCode::NOT_FOUND => Ok(false),
            status
                if status == reqwest::StatusCode::FORBIDDEN
                    || status == reqwest::StatusCode::UNAUTHORIZED =>
            {
                Err(VersionControlClientBranchCheckError::Unauthorized(format!(
                    "GitHub returned {}",
                    status
                )))
            }
            status => Err(VersionControlClientBranchCheckError::Transport(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }
}
