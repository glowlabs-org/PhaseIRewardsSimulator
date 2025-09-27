use crate::errors::SimError;
use crate::models::{FarmReward, OutputData};
use crate::simulator::{
    simulate_with_diagnostics, DetailedBucket, DetailedCompetition, DetailedFarmBucketState,
    DetailedFarmInfo, SimulationDiagnostics,
};
use axum::extract::{Path, Query};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
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
        .route("/harness.js", get(harness_js_handler))
        .route("/tests.js", get(tests_js_handler))
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
    axum::extract::Json(input): axum::extract::Json<crate::models::InputData>,
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
    axum::extract::Json(input): axum::extract::Json<crate::models::InputData>,
) -> Result<Response, AppError> {
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            let want_strings = q.as_strings.unwrap_or(false);
            if want_strings {
                let status = if diag.errors.is_empty() {
                    StatusCode::OK
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                let sdiag = stringify_diagnostics(diag);
                Ok((status, axum::Json(sdiag)).into_response())
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

// ---------- Stringified diagnostics (for frontend BigInt-safety) ----------

#[derive(Clone, Debug, Serialize)]
struct SimulationDiagnosticsStrings {
    output: OutputDataStrings,
    errors: Vec<String>,
    competitions: Vec<DetailedCompetitionStrings>,
}

#[derive(Clone, Debug, Serialize)]
struct OutputDataStrings {
    total_regions: usize,
    regional_stats: Vec<crate::models::RegionStats>,
    weekly_rewards: Vec<WeekRewardsStrings>,
}

#[derive(Clone, Debug, Serialize)]
struct WeekRewardsStrings {
    week_number: u64,
    per_farm_rewards: Vec<FarmRewardStrings>,
}

#[derive(Clone, Debug, Serialize)]
struct FarmRewardStrings {
    farm_id: String,
    asset_id: String,
    region_id: String,
    amount: String,
    rewards_address: String,
}

#[derive(Clone, Debug, Serialize)]
struct DetailedCompetitionStrings {
    region_id: String,
    asset_id: String,
    first_week: u64,
    final_week: u64,
    farms: Vec<DetailedFarmInfoStrings>,
    buckets: Vec<DetailedBucketStrings>,
}

#[derive(Clone, Debug, Serialize)]
struct DetailedFarmInfoStrings {
    farm_id: String,
    protocol_deposit_value: String,
    assets_required: String,
    first_week: u64,
    final_week: u64,
    rewards_address: String,
    asset_id: String,
    region_id: String,
}

#[derive(Clone, Debug, Serialize)]
struct DetailedBucketStrings {
    week_number: u64,
    total_deposits: String,
    total_carbon_credits: String,
    pool_net_assets: String,
    pool_net_deposits: String,
    first_week_farms: Vec<String>,
    ongoing_farms: Vec<String>,
    last_week_farms: Vec<String>,
    farm_states: Vec<DetailedFarmBucketStateStrings>,
}

#[derive(Clone, Debug, Serialize)]
struct DetailedFarmBucketStateStrings {
    farm_id: String,
    deposits_contributed: String,
    carbon_credits_contributed: String,
    accumulated_drawdown: String,
    net_overperformance: String,
    rewards_this_week: String,
}

fn stringify_diagnostics(diag: SimulationDiagnostics) -> SimulationDiagnosticsStrings {
    SimulationDiagnosticsStrings {
        output: stringify_output(&diag.output),
        errors: diag.errors,
        competitions: diag.competitions.into_iter().map(stringify_comp).collect(),
    }
}

fn stringify_output(out: &OutputData) -> OutputDataStrings {
    OutputDataStrings {
        total_regions: out.total_regions,
        regional_stats: out.regional_stats.clone(),
        weekly_rewards: out
            .weekly_rewards
            .iter()
            .map(|w| WeekRewardsStrings {
                week_number: w.week_number,
                per_farm_rewards: stringify_farm_rewards(&w.per_farm_rewards),
            })
            .collect(),
    }
}

fn stringify_farm_rewards(items: &[FarmReward]) -> Vec<FarmRewardStrings> {
    items
        .iter()
        .map(|fr| FarmRewardStrings {
            farm_id: fr.farm_id.clone(),
            asset_id: fr.asset_id.clone(),
            region_id: fr.region_id.clone(),
            amount: fr.amount.to_string(),
            rewards_address: fr.rewards_address.clone(),
        })
        .collect()
}

fn stringify_comp(dc: DetailedCompetition) -> DetailedCompetitionStrings {
    DetailedCompetitionStrings {
        region_id: dc.region_id,
        asset_id: dc.asset_id,
        first_week: dc.first_week,
        final_week: dc.final_week,
        farms: dc.farms.into_iter().map(stringify_farm_info).collect(),
        buckets: dc.buckets.into_iter().map(stringify_bucket).collect(),
    }
}

fn stringify_farm_info(df: DetailedFarmInfo) -> DetailedFarmInfoStrings {
    DetailedFarmInfoStrings {
        farm_id: df.farm_id,
        protocol_deposit_value: df.protocol_deposit_value.to_string(),
        assets_required: df.assets_required.to_string(),
        first_week: df.first_week,
        final_week: df.final_week,
        rewards_address: df.rewards_address,
        asset_id: df.asset_id,
        region_id: df.region_id,
    }
}

fn stringify_bucket(db: DetailedBucket) -> DetailedBucketStrings {
    DetailedBucketStrings {
        week_number: db.week_number,
        total_deposits: db.total_deposits.to_string(),
        total_carbon_credits: db.total_carbon_credits.to_string(),
        pool_net_assets: db.pool_net_assets.to_string(),
        pool_net_deposits: db.pool_net_deposits.to_string(),
        first_week_farms: db.first_week_farms,
        ongoing_farms: db.ongoing_farms,
        last_week_farms: db.last_week_farms,
        farm_states: db
            .farm_states
            .into_iter()
            .map(stringify_farm_state)
            .collect(),
    }
}

fn stringify_farm_state(st: DetailedFarmBucketState) -> DetailedFarmBucketStateStrings {
    DetailedFarmBucketStateStrings {
        farm_id: st.farm_id,
        deposits_contributed: st.deposits_contributed.to_string(),
        carbon_credits_contributed: st.carbon_credits_contributed.to_string(),
        accumulated_drawdown: st.accumulated_drawdown.to_string(),
        net_overperformance: st.net_overperformance.to_string(),
        rewards_this_week: st.rewards_this_week.to_string(),
    }
}
