use crate::config::environment::ENV;
use crate::server::webhook::github::controller::GithubHeaders;
use axum::body::{to_bytes, Body, Bytes};
use axum::http::StatusCode;
use axum::{extract::Request, middleware::Next, response::Response};
use hmac::Hmac;
use sha2::digest::Mac;
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

pub async fn handle(mut request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if true {
        return Ok(next.run(request).await);
    }

    let secret = ENV.get("GITHUB_WEBHOOK_SECRET");

    let signature = request
        .headers()
        .get(GithubHeaders::SIGNATURE_256)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let signature = signature.strip_prefix("sha256=").unwrap_or(signature);

    // Декодируем hex безопасно
    let signature_bytes = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(error = ?err, "Invalid hex in GitHub signature: {}", signature);
            return Err(StatusCode::FORBIDDEN);
        }
    };

    // Создаём HMAC
    let mut hmac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(hmac) => hmac,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to create HMAC with secret");
            return Err(StatusCode::FORBIDDEN);
        }
    };

    let (parts, body) = request.into_parts();

    let payload: Bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to convert request body to bytes");
            return Err(StatusCode::FORBIDDEN);
        }
    };

    // Вычисляем HMAC
    hmac.update(&*payload);

    // Проверяем подпись
    let verified = match hmac.verify_slice(&signature_bytes) {
        Ok(_) => true,
        Err(e) => {
            tracing::error!(error = ?e, "GitHub webhook signature verification failed");
            false
        }
    };

    if (!verified) {
        return Err(StatusCode::FORBIDDEN);
    }

    let body = Body::from(payload);
    let request = Request::from_parts(parts, body);

    Ok(next.run(request).await)
}
