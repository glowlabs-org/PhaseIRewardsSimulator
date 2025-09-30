use crate::competition_simulator::simulate_with_diagnostics;
use crate::errors::SimError;
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::json;
use std::fs as stdfs;
use std::path::{Path as FsPath, PathBuf};

#[derive(Debug, Deserialize)]
pub struct SimQuery {
    #[serde(rename = "preloadGlowV1")]
    preload_glow_v1: Option<String>,
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/styles.css", get(styles_handler))
        .route("/app.js", get(js_handler))
        .route("/harness.js", get(harness_js_handler))
        .route("/tests.js", get(tests_js_handler))
        .route("/assets/*path", get(assets_handler))
        .route("/api/rewards-simulator", post(sim_handler))
        .route(
            "/api/rewards-simulator-detailed",
            post(sim_detailed_handler),
        )
        .route("/ui/rewards-simulator-detailed", post(sim_detailed_handler))
}

async fn sim_handler(
    Query(query): Query<SimQuery>,
    axum::extract::Json(mut input): axum::extract::Json<crate::models::InputData>,
) -> Result<Response, AppError> {
    if query.preload_glow_v1.as_deref() == Some("true") {
        input = crate::preload::merge_with_v1_data(input)?;
    }
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            if diag.errors.is_empty() {
                Ok((StatusCode::OK, axum::Json(diag.output)).into_response())
            } else {
                let body = json!({ "errors": diag.errors, "output": diag.output });
                Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response())
            }
        }
        Err(e) => Err(AppError(e)),
    }
}

async fn sim_detailed_handler(
    Query(query): Query<SimQuery>,
    axum::extract::Json(mut input): axum::extract::Json<crate::models::InputData>,
) -> Result<Response, AppError> {
    if query.preload_glow_v1.as_deref() == Some("true") {
        input = crate::preload::merge_with_v1_data(input)?;
    }
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            if diag.errors.is_empty() {
                Ok((StatusCode::OK, axum::Json(diag)).into_response())
            } else {
                Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(diag)).into_response())
            }
        }
        Err(e) => Err(AppError(e)),
    }
}

#[derive(Debug)]
pub struct AppError(pub SimError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = self.0.to_string();
        let status = match self.0 {
            SimError::Validation(_) => StatusCode::BAD_REQUEST,
            SimError::Algorithm(_) => StatusCode::UNPROCESSABLE_ENTITY,
            SimError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = axum::Json(serde_json::json!({ "error": msg }));
        (status, body).into_response()
    }
}

impl From<SimError> for AppError {
    fn from(value: SimError) -> Self {
        AppError(value)
    }
}

const INDEX_HTML: &str = include_str!("web/index.html");
const STYLES_CSS: &str = include_str!("web/styles.css");
const APP_JS: &str = include_str!("web/app.js");
const HARNESS_JS: &str = include_str!("web/harness.js");
const TESTS_JS: &str = include_str!("web/tests.js");

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

async fn styles_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STYLES_CSS,
    )
}

async fn js_handler() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        APP_JS,
    )
}
async fn harness_js_handler() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        HARNESS_JS,
    )
}
async fn tests_js_handler() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        TESTS_JS,
    )
}

async fn assets_handler(Path(path): Path<String>) -> impl IntoResponse {
    if let Some(pb) = sanitize_asset_path(&path) {
        let base: PathBuf = ["src", "web", "assets"].iter().collect();
        let full = base.join(pb);
        if full.exists() && full.is_file() {
            match stdfs::read(&full) {
                Ok(bytes) => {
                    let ct = content_type_for(&full);
                    return ([(header::CONTENT_TYPE, ct)], bytes).into_response();
                }
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

fn sanitize_asset_path(s: &str) -> Option<PathBuf> {
    if s.is_empty() {
        return None;
    }
    if s.contains("..") || s.contains('\\') {
        return None;
    }
    let pb = FsPath::new(s);
    let mut out = PathBuf::new();
    for comp in pb.components() {
        match comp {
            std::path::Component::Normal(seg) => out.push(seg),
            _ => return None,
        }
    }
    Some(out)
}

fn content_type_for(p: &FsPath) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or_default() {
        "svg" => "image/svg+xml",
        "otf" => "font/otf",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}
