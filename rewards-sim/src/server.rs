use crate::competition_simulator::{build_public_output_from_detailed, simulate_with_diagnostics};
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
    week: Option<String>,
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/styles.css", get(styles_handler))
        .route("/harness.js", get(harness_js_handler))
        .route("/tests.js", get(tests_js_handler))
        .route("/assets/*path", get(assets_handler))
        .route("/js/*path", get(js_handler_dynamic))
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
        input = crate::preload::load_and_merge_v1_data(input)?;
    }
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            let public_out = build_public_output_from_detailed(&diag.competitions, &diag.errors);
            if let Some(week_str) = query.week.as_ref().and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }) {
                let key = week_str
                    .parse::<u64>()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|_| week_str.clone());

                if let Some(obj) = public_out.get(&key) {
                    if diag.errors.is_empty() {
                        Ok((StatusCode::OK, axum::Json(obj)).into_response())
                    } else {
                        let body = json!({ "errors": diag.errors, "output": obj });
                        Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response())
                    }
                } else {
                    let body = axum::Json(serde_json::json!({
                        "error": format!("requested week not found: {}", key)
                    }));
                    Ok((StatusCode::NOT_FOUND, body).into_response())
                }
            } else if diag.errors.is_empty() {
                Ok((StatusCode::OK, axum::Json(public_out)).into_response())
            } else {
                let body = json!({ "errors": diag.errors, "output": public_out });
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
        input = crate::preload::load_and_merge_v1_data(input)?;
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

async fn js_handler_dynamic(Path(path): Path<String>) -> impl IntoResponse {
    if let Some(pb) = sanitize_asset_path(&path) {
        let base: PathBuf = ["src", "web", "js"].iter().collect();
        let full = base.join(pb);
        if full.exists() && full.is_file() {
            match stdfs::read(&full) {
                Ok(bytes) => {
                    let ct = "application/javascript; charset=utf-8";
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
        "js" => "application/javascript; charset=utf-8",
        _ => "application/octet-stream",
    }
}
