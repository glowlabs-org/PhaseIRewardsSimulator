use crate::competition_simulator::{
    build_public_output_from_detailed, simulate_multi_asset, simulate_with_diagnostics,
};
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
        .route(
            "/api/rewards-simulator-multi-asset",
            post(sim_multi_asset_handler),
        )
}

async fn sim_handler(
    Query(query): Query<SimQuery>,
    axum::extract::Json(mut input): axum::extract::Json<crate::models::InputData>,
) -> Result<Response, AppError> {
    if query.preload_glow_v1.as_deref() == Some("true") {
        input = crate::preload::load_and_merge_v1_data(input)?;
    }
    let output_farms = input.output_farms.clone();

    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            let public_out = build_public_output_from_detailed(&diag.competitions, &diag.errors);

            let final_output = if let Some(farm_ids) = output_farms {
                let farm_id_set: std::collections::HashSet<String> = farm_ids.into_iter().collect();
                let mut reduced_out_map: serde_json::Map<String, serde_json::Value> =
                    serde_json::Map::new();
                for (week, mut week_output) in public_out {
                    week_output
                        .farm_rewards
                        .retain(|fr| farm_id_set.contains(&fr.id));
                    let reduced_week = json!({
                        "farmRewards": week_output.farm_rewards,
                        "warnings": week_output.warnings,
                    });
                    reduced_out_map.insert(week, reduced_week);
                }
                serde_json::Value::Object(reduced_out_map)
            } else {
                serde_json::to_value(public_out)
                    .map_err(|e| AppError(SimError::internal(e.to_string())))?
            };

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

                if let Some(obj) = final_output.get(&key) {
                    if diag.errors.is_empty() {
                        Ok((StatusCode::OK, axum::Json(obj.clone())).into_response())
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
                Ok((StatusCode::OK, axum::Json(final_output)).into_response())
            } else {
                let body = json!({ "errors": diag.errors, "output": final_output });
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

async fn sim_multi_asset_handler(
    Query(query): Query<SimQuery>,
    axum::extract::Json(input): axum::extract::Json<crate::models::InputDataMultiAsset>,
) -> Result<Response, AppError> {
    let preload = query.preload_glow_v1.as_deref() == Some("true");
    let has_output_filter = input.output_farms.is_some();
    match simulate_multi_asset(input, preload) {
        Ok((out_map, errors)) => {
            // Apply week filter if needed
            let final_output = if let Some(week_str) = query.week.as_ref().and_then(|s| {
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

                if let Some(obj) = out_map.get(&key) {
                    serde_json::to_value(obj)
                        .map_err(|e| AppError(SimError::internal(e.to_string())))?
                } else {
                    let body = axum::Json(serde_json::json!({
                        "error": format!("requested week not found: {}", key)
                    }));
                    return Ok((StatusCode::NOT_FOUND, body).into_response());
                }
            } else {
                serde_json::to_value(out_map)
                    .map_err(|e| AppError(SimError::internal(e.to_string())))?
            };

            // If outputFarms was specified, we need to ensure the response structure
            // only contains farmRewards and warnings.
            let final_output = if has_output_filter {
                if let serde_json::Value::Object(mut map) = final_output {
                    // It could be a single week object or the full map.
                    // If it's the full map (keys are week numbers), iterate values.
                    // If it's a single week object (keys are "farmRewards" etc), filter directly.
                    if map.contains_key("farmRewards") {
                        // Single week
                        map.remove("walletDistributions");
                        map.remove("regionData");
                        serde_json::Value::Object(map)
                    } else {
                        // Full map
                        for (_, val) in map.iter_mut() {
                            if let serde_json::Value::Object(w_obj) = val {
                                w_obj.remove("walletDistributions");
                                w_obj.remove("regionData");
                            }
                        }
                        serde_json::Value::Object(map)
                    }
                } else {
                    final_output
                }
            } else {
                final_output
            };

            if errors.is_empty() {
                Ok((StatusCode::OK, axum::Json(final_output)).into_response())
            } else {
                let body = json!({ "errors": errors, "output": final_output });
                Ok((StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response())
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
