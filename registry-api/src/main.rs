//! Read-only HTTP JSON API for the LP-0023 LEZ Program Registry.
//!
//! Serves the on-chain global index and decoded entries over GET endpoints.
//! Configuration is entirely via environment variables (see README.md).
//! Dual-licensed: MIT OR Apache-2.0.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use nssa_core::program::ProgramId;
use registry_api::{
    filter_entries, parse_key32, program_id_from_hex, to_base58, RegistryService,
    DEFAULT_PROGRAM_ID_HEX, DEFAULT_WALLET_HOME,
};
use registry_core::RegistryEntry;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

/// Address the HTTP server binds to by default.
const DEFAULT_ADDR: &str = "127.0.0.1:3030";

#[derive(Clone)]
struct ApiState {
    registry: RegistryService,
    program_id: ProgramId,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: message.into() }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, message: message.into() }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: message.into() }
    }

    fn bad_gateway(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_GATEWAY, message: message.into() }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorBody { error: self.message })).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

struct SearchParams {
    name: Option<String>,
    tags: Vec<String>,
    author: Option<[u8; 32]>,
    verified_only: bool,
    limit: Option<usize>,
}

/// Parse the query string manually so repeated `tag` keys work and every bad
/// parameter produces a JSON 400 (instead of axum's plain-text rejection).
fn parse_search_params(raw: Option<String>) -> ApiResult<SearchParams> {
    let mut name = None;
    let mut tags = Vec::new();
    let mut author = None;
    let mut verified = None;
    let mut limit = None;

    for (key, value) in form_urlencoded::parse(raw.as_deref().unwrap_or("").as_bytes()) {
        match key.as_ref() {
            "name" => {
                if name.is_some() {
                    return Err(ApiError::bad_request("name may be specified only once"));
                }
                if value.is_empty() {
                    return Err(ApiError::bad_request("name must not be empty"));
                }
                name = Some(value.into_owned());
            }
            "tag" => {
                if value.is_empty() {
                    return Err(ApiError::bad_request("tag must not be empty"));
                }
                tags.push(value.into_owned());
            }
            "author" => {
                if author.is_some() {
                    return Err(ApiError::bad_request("author may be specified only once"));
                }
                author = Some(
                    parse_key32(&value)
                        .map_err(|e| ApiError::bad_request(format!("invalid author: {}", e)))?,
                );
            }
            "verified" => {
                if verified.is_some() {
                    return Err(ApiError::bad_request("verified may be specified only once"));
                }
                verified = Some(parse_bool(&value)?);
            }
            "limit" => {
                if limit.is_some() {
                    return Err(ApiError::bad_request("limit may be specified only once"));
                }
                limit = Some(value.parse::<usize>().map_err(|_| {
                    ApiError::bad_request(format!("invalid limit: '{}' is not a non-negative integer", value))
                })?);
            }
            other => return Err(ApiError::bad_request(format!("unknown query parameter: '{}'", other))),
        }
    }

    Ok(SearchParams {
        name,
        tags,
        author,
        verified_only: verified.unwrap_or(false),
        limit,
    })
}

fn parse_bool(value: &str) -> ApiResult<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(ApiError::bad_request(format!("invalid verified value: '{}' (expected true or false)", value))),
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn index(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vec<String>>> {
    let index = state.registry.index(state.program_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(index.program_ids.iter().map(to_base58).collect()))
}

async fn entries(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vec<RegistryEntry>>> {
    let index = state.registry.index(state.program_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let entries = state.registry.entries(state.program_id, index.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(entries))
}

async fn search(
    State(state): State<Arc<ApiState>>,
    raw_query: RawQuery,
) -> ApiResult<Json<Vec<RegistryEntry>>> {
    let params = parse_search_params(raw_query.0)?;
    let index = state.registry.index(state.program_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let entries = state.registry.entries(state.program_id, index.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let matched = filter_entries(
        &entries,
        &params.tags,
        params.author.as_ref(),
        params.verified_only,
        params.name.as_deref(),
    );
    let matched: Vec<RegistryEntry> = matched
        .into_iter()
        .take(params.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect();
    Ok(Json(matched))
}

async fn entry(
    State(state): State<Arc<ApiState>>,
    Path(program_id): Path<String>,
) -> ApiResult<Json<RegistryEntry>> {
    let key = parse_key32(&program_id)
        .map_err(|e| ApiError::bad_request(format!("invalid program id: {}", e)))?;
    let index = state.registry.index(state.program_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !index.program_ids.contains(&key) {
        return Err(ApiError::not_found(format!("program id '{}' is not in the registry index", program_id)));
    }
    let entry = state.registry.entry(state.program_id, key)
        .await
        .map_err(|e| ApiError::bad_gateway(format!("failed to load entry '{}': {}", program_id, e)))?;
    Ok(Json(entry))
}

/// Make sure `WalletCore::from_env()` uses the documented default home.
fn configure_wallet_home() -> PathBuf {
    if let Some(home) = env::var_os("LEE_WALLET_HOME_DIR") {
        return PathBuf::from(home);
    }
    let home = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(DEFAULT_WALLET_HOME);
    env::set_var("LEE_WALLET_HOME_DIR", &home);
    home
}

#[tokio::main]
async fn main() {
    let addr = env::var("REGISTRY_API_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let program_id_hex = env::var("REGISTRY_PROGRAM_ID")
        .unwrap_or_else(|_| DEFAULT_PROGRAM_ID_HEX.to_string());
    let program_id = program_id_from_hex(&program_id_hex).unwrap_or_else(|e| {
        eprintln!("error: invalid REGISTRY_PROGRAM_ID: {}", e);
        std::process::exit(1);
    });

    let wallet_home = configure_wallet_home();
    eprintln!("registry program id : 0x{}", program_id_hex.trim_start_matches("0x"));
    eprintln!("wallet home         : {}", wallet_home.display());

    let registry = RegistryService::start().await.unwrap_or_else(|e| {
        eprintln!("error: failed to init wallet: {}", e);
        eprintln!("set LEE_WALLET_HOME_DIR to the registry wallet home");
        std::process::exit(1);
    });

    let state = Arc::new(ApiState { registry, program_id });
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/index", get(index))
        .route("/entries", get(entries))
        .route("/search", get(search))
        .route("/entry/:program_id", get(entry))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("error: failed to bind {}: {}", addr, e);
        std::process::exit(1);
    });
    eprintln!("registry-api listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("error: server failed: {}", e);
        std::process::exit(1);
    });
}
