use crate::applications::state::ApplicationState;
use crate::config::environment::ENV;
use crate::domain::user::use_cases::create::CreateUserUseCase;
use crate::infrastructure::repository::mysql::user::MySQLUserRepository;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use std::sync::Arc;

async fn handle(
    State(state): State<Arc<ApplicationState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let repo = MySQLUserRepository { db: state.db.clone() };

    let use_case = CreateUserUseCase::new(repo);

    use_case.execute().await.unwrap();

    Json(payload)
}

pub fn create() -> Router<Arc<ApplicationState>> {
    let secret = ENV.get_or("GITHUB_WEBHOOK_SECRET", "");

    if secret == "" {
        tracing::warn!(
            "!!! GITHUB_WEBHOOK_SECRET is empty. If you use this in production more please, add this environment !!!"
        );
    }

    Router::new().route("/webhook/github", post(handle))
}