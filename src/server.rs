use crate::errors::SimError;
use crate::models::InputData;
use crate::simulator::{simulate_with_diagnostics, SimulationDiagnostics};
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::json;
use std::fs as stdfs;
use std::path::{Path as FsPath, PathBuf};

pub fn app() -> Router {
    Router::new()
        // UI
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/styles.css", get(styles_handler))
        .route("/app.js", get(js_handler))
        .route("/assets/*path", get(assets_handler))
        // API
        .route("/api/rewards-simulator", post(sim_handler))
        .route(
            "/api/rewards-simulator-detailed",
            post(sim_detailed_handler),
        )
        // Optional: alias for UI usage
        .route("/ui/rewards-simulator-detailed", post(sim_detailed_handler))
}

async fn sim_handler(
    axum::extract::Json(input): axum::extract::Json<InputData>,
) -> Result<Response, AppError> {
    match simulate_with_diagnostics(input) {
        Ok(SimulationDiagnostics { output, errors, .. }) => {
            if errors.is_empty() {
                Ok((StatusCode::OK, axum::Json(output)).into_response())
            } else {
                let body = json!({ "errors": errors, "output": output });
                Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response())
            }
        }
        Err(e) => Err(AppError(e)),
    }
}

#[derive(Deserialize)]
struct AsStrings {
    as_strings: Option<bool>,
}

async fn sim_detailed_handler(
    Query(q): Query<AsStrings>,
    axum::extract::Json(input): axum::extract::Json<InputData>,
) -> Result<Response, AppError> {
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            if q.as_strings.unwrap_or(false) {
                if diag.errors.is_empty() {
                    Ok((StatusCode::OK, axum::Json(diag)).into_response())
                } else {
                    Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(diag)).into_response())
                }
            } else if diag.errors.is_empty() {
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

// ---------- Static UI handlers (embedded html/css/js) ----------

const INDEX_HTML: &str = include_str!("web/index.html");
const STYLES_CSS: &str = include_str!("web/styles.css");
const APP_JS: &str = include_str!("web/app.js");

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

// Serve assets (fonts, etc.) from src/web/assets/ at runtime.
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
