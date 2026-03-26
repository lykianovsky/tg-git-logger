use crate::domain::shared::date::range::DateRange;
use crate::domain::version_control::ports::version_control_client::{
    VersionControlClient, VersionControlClientDateRangeReportError,
    VersionControlClientGetUserError, VersionControlClientGetUserResponse,
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
    query_path = "src/infrastructure/integrations/version_control/github/graphql/queries/get_date_range_report.graphql"
)]
pub struct GithubDateRangeReport;

pub struct GithubVersionControlClient {
    base: String,
    owner: String,
    repository: String,
    client: Client,
}

impl GithubVersionControlClient {
    pub fn new(base: String, owner: String, repository: String) -> Self {
        Self {
            base,
            owner,
            repository,
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
        let author_filter = author.map(|a| format!(" author:{}", a)).unwrap_or_default();

        let pr_search = format!(
            "repo:{}/{} is:pr created:{}..{}{}",
            owner,
            repo,
            date_range.since.format("%Y-%m-%d"),
            date_range.until.format("%Y-%m-%d"),
            author_filter,
        );
        let issue_search = format!(
            "repo:{}/{} is:issue created:{}..{}{}",
            owner,
            repo,
            date_range.since.format("%Y-%m-%d"),
            date_range.until.format("%Y-%m-%d"),
            author_filter,
        );

        let response = self
            .graphql::<GithubDateRangeReport>(
                access_token,
                github_date_range_report::Variables {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    branch: branch.to_string(),
                    since: date_range.since,
                    until: date_range.until,
                    pr_search,
                    issue_search: issue_search.to_string(),
                    after: None,
                },
            )
            .await
            .map_err(|e| {
                if let Some(graphql_error) = e.get_error_by_type(GithubGraphQLErrorType::Forbidden)
                {
                    return VersionControlClientDateRangeReportError::Unauthorized(
                        graphql_error.message.to_string(),
                    );
                }

                VersionControlClientDateRangeReportError::Transport(e.to_string())
            })?;

        // If the repository exists but the ref is None, the branch does not exist
        if response.repository.is_some()
            && response
                .repository
                .as_ref()
                .and_then(|r| r.ref_.as_ref())
                .is_none()
        {
            return Err(VersionControlClientDateRangeReportError::BranchNotFound(
                branch.to_string(),
            ));
        }

        Ok(VersionControlDateRangeReport::from_github_response(
            response,
        ))
    }
}

impl VersionControlDateRangeReport {
    pub fn from_github_response(response: github_date_range_report::ResponseData) -> Self {
        let mut pull_requests = Vec::new();
        let mut commits = Vec::new();

        if let Some(pr_nodes) = response.pull_requests.nodes {
            for node in pr_nodes.into_iter().flatten() {
                if let github_date_range_report::GithubDateRangeReportPullRequestsNodes::PullRequest(pr) =
                    node
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

        if let Some(target) = response
            .repository
            .and_then(|r| r.ref_)
            .and_then(|b| b.target)
            && let github_date_range_report::GithubDateRangeReportRepositoryRefTarget::Commit(
                commit_target,
            ) = target
            && let Some(nodes) = commit_target.history.nodes
        {
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

        VersionControlDateRangeReport {
            pull_requests,
            commits,
        }
    }
}
