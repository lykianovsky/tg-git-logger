use crate::infrastructure::contracts::github::headers::GithubHeaders;
use axum::body::{Body, Bytes, to_bytes};
use axum::http::StatusCode;
use axum::{extract::Request, middleware::Next, response::Response};
use hmac::Hmac;
use sha2::Sha256;
use sha2::digest::Mac;

const MAX_WEBHOOK_BODY_BYTES: usize = 10 * 1024 * 1024;

pub struct GithubWebhookAuthorizationMiddleware {
    secret: String,
}

type HmacSha256 = Hmac<Sha256>;

impl GithubWebhookAuthorizationMiddleware {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub async fn handle(self, request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
        if self.secret.is_empty() {
            tracing::error!(
                "GITHUB_WEBHOOK_SECRET is not configured; rejecting webhook request"
            );
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        let signature = request
            .headers()
            .get(GithubHeaders::SIGNATURE_256)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let signature = signature.strip_prefix("sha256=").unwrap_or(signature);

        let signature_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => {
                tracing::warn!("Invalid hex in GitHub signature header");
                return Err(StatusCode::FORBIDDEN);
            }
        };

        let mut hmac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(hmac) => hmac,
            Err(err) => {
                tracing::error!(error = ?err, "Failed to create HMAC with secret");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let (parts, body) = request.into_parts();

        let payload: Bytes = match to_bytes(body, MAX_WEBHOOK_BODY_BYTES).await {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::warn!(error = ?err, "Webhook body too large or read failed");
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        };

        hmac.update(&payload);

        if hmac.verify_slice(&signature_bytes).is_err() {
            tracing::warn!("GitHub webhook signature verification failed");
            return Err(StatusCode::FORBIDDEN);
        }

        let body = Body::from(payload);
        let request = Request::from_parts(parts, body);

        Ok(next.run(request).await)
    }
}
