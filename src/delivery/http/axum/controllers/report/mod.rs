use axum::Extension;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use crate::bootstrap::shared_dependency::ApplicationSharedDependency;

pub struct AxumReportController;

impl AxumReportController {
    pub async fn handle_get(
        Path(token): Path<String>,
        Extension(shared): Extension<Arc<ApplicationSharedDependency>>,
    ) -> Response {
        let key = format!("user_report_html:{}", token);

        match shared.cache.get(&key).await {
            Ok(Some(html)) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            )
                .into_response(),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                r#"<!DOCTYPE html><html lang="ru"><head><meta charset="UTF-8">
<title>Отчёт не найден</title>
<style>
  body{font-family:sans-serif;background:#0d1117;color:#c9d1d9;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
  .box{text-align:center;padding:40px}
  h2{color:#8b949e;font-size:18px;font-weight:400;margin-top:8px}
  p{color:#6e7681;font-size:14px;margin-top:8px}
  .icon{font-size:64px}
</style></head>
<body><div class="box">
<div class="icon">🔭</div>
<h2>Отчёт не найден или устарел</h2>
<p>Отчёты хранятся 10 минут. Сформируйте новый отчёт в боте.</p>
</div></body></html>"#
                    .to_string(),
            )
                .into_response(),
            Err(e) => {
                tracing::error!(error = %e, token = %token, "Failed to retrieve report html from cache");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    "Internal server error".to_string(),
                )
                    .into_response()
            }
        }
    }
}
