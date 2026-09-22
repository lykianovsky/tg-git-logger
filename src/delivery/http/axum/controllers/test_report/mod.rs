use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::infrastructure::services::test_report::local::TEST_REPORT_TOKEN_CACHE_PREFIX;
use axum::Extension;
use axum::extract::Path as AxumPath;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

const DEFAULT_REPORT_FILE: &str = "index.html";

pub struct AxumTestReportController;

impl AxumTestReportController {
    /// Корень отчёта: `/test-report/{token}/`
    pub async fn handle_root(
        AxumPath(token): AxumPath<String>,
        shared: Extension<Arc<ApplicationSharedDependency>>,
        config: Extension<Arc<ApplicationConfig>>,
    ) -> Response {
        Self::serve(token, DEFAULT_REPORT_FILE.to_string(), shared, config).await
    }

    /// Ассеты отчёта: `/test-report/{token}/{*path}`
    pub async fn handle_asset(
        AxumPath((token, path)): AxumPath<(String, String)>,
        shared: Extension<Arc<ApplicationSharedDependency>>,
        config: Extension<Arc<ApplicationConfig>>,
    ) -> Response {
        Self::serve(token, path, shared, config).await
    }

    async fn serve(
        token: String,
        path: String,
        Extension(shared): Extension<Arc<ApplicationSharedDependency>>,
        Extension(config): Extension<Arc<ApplicationConfig>>,
    ) -> Response {
        let run_id = match shared
            .cache
            .get(&format!("{TEST_REPORT_TOKEN_CACHE_PREFIX}{token}"))
            .await
        {
            Ok(Some(value)) => value,
            Ok(None) => return Self::expired_page(),
            Err(error) => {
                tracing::error!(%error, "Failed to read test report token");

                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    .into_response();
            }
        };

        let run_dir = PathBuf::from(&config.test_control.reports_dir).join(&run_id);

        // Токен даёт доступ только к каталогу своего прогона: `..` в пути отклоняем
        let Some(file_path) = Self::resolve_path(&run_dir, &path) else {
            return (StatusCode::NOT_FOUND, "Not found").into_response();
        };

        match tokio::fs::read(&file_path).await {
            Ok(body) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, Self::content_type(&file_path))],
                body,
            )
                .into_response(),
            Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
        }
    }

    fn resolve_path(run_dir: &Path, path: &str) -> Option<PathBuf> {
        let relative = PathBuf::from(path);

        if relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return None;
        }

        Some(run_dir.join(relative))
    }

    fn content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|value| value.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("json") => "application/json; charset=utf-8",
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("webm") => "video/webm",
            Some("zip") => "application/zip",
            Some("woff2") => "font/woff2",
            _ => "application/octet-stream",
        }
    }

    fn expired_page() -> Response {
        (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            r#"<!DOCTYPE html><html lang="ru"><head><meta charset="UTF-8">
<title>Отчёт недоступен</title>
<style>
  body{font-family:sans-serif;background:#0d1117;color:#c9d1d9;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}
  .box{text-align:center;padding:40px}
  h2{color:#8b949e;font-size:18px;font-weight:400;margin-top:8px}
  p{color:#6e7681;font-size:14px;margin-top:8px}
  .icon{font-size:64px}
</style></head>
<body><div class="box">
<div class="icon">🧪</div>
<h2>Ссылка на отчёт устарела</h2>
<p>Запросите отчёт в боте заново.</p>
</div></body></html>"#,
        )
            .into_response()
    }
}
