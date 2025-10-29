use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_tempfile::TempFile;
use axum::{
    extract::DefaultBodyLimit,
    extract::{multipart::MultipartError, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, get_service, post},
    Json, Router,
};
use price_engine::{EngineError, ModelKind, PriceEngine, QuoteInput, QuoteOutput};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use tower_http::{
    cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{error, info};

const DEFAULT_MAX_UPLOAD_MB: usize = 50;
const DEFAULT_MATERIAL_ID: &str = "pla";
const DEFAULT_INFILL_PERCENT: u8 = 25;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();

    let state = AppState::initialize(PriceEngine::new())?;
    let max_upload_bytes = state.max_upload_bytes;
    let app = router(state.clone(), max_upload_bytes);

    let addr: SocketAddr = std::env::var("RAPIDMAKER_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

fn router(state: AppState, max_upload_bytes: usize) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/materials", get(list_materials))
        .route("/api/quote/gcode", post(quote_gcode))
        .route("/api/quote", post(quote_file))
        .fallback_service(get_service(
            ServeDir::new("frontend/dist").append_index_html_on_directories(true),
        ))
        .with_state(state)
        .layer(DefaultBodyLimit::max(max_upload_bytes))
        .layer(RequestBodyLimitLayer::new(max_upload_bytes))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

fn setup_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".to_string());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("unable to set tracing subscriber");
}

#[derive(Clone)]
struct AppState {
    engine: Arc<PriceEngine>,
    max_upload_bytes: usize,
    upload_dir: Arc<PathBuf>,
}

impl AppState {
    fn initialize(engine: PriceEngine) -> Result<Self, std::io::Error> {
        let max_upload_mb = std::env::var("RAPIDMAKER_MAX_UPLOAD_MB")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&mb| mb > 0)
            .unwrap_or(DEFAULT_MAX_UPLOAD_MB);

        let upload_dir = std::env::var("RAPIDMAKER_UPLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("rapidmaker-uploads"));

        std::fs::create_dir_all(&upload_dir)?;

        Ok(Self {
            engine: Arc::new(engine),
            max_upload_bytes: max_upload_mb * 1024 * 1024,
            upload_dir: Arc::new(upload_dir),
        })
    }
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        max_upload_bytes: state.max_upload_bytes,
    })
}

async fn list_materials(State(state): State<AppState>) -> Json<Vec<MaterialDTO>> {
    let items = state
        .engine
        .materials()
        .map(|(_, profile)| MaterialDTO::from(profile.clone()))
        .collect();
    Json(items)
}

async fn quote_gcode(
    State(state): State<AppState>,
    Json(payload): Json<GcodeQuoteRequest>,
) -> Result<Json<QuoteOutput>, ApiError> {
    let mut input = QuoteInput {
        material_id: payload.material_id.clone(),
        infill_percent: Some(DEFAULT_INFILL_PERCENT),
        supports: true,
        cost_per_kg_override: None,
        base_fee_override: None,
    };

    if input.material_id.is_empty() {
        input.material_id = DEFAULT_MATERIAL_ID.to_string();
    }

    let output = state
        .engine
        .quote_from_gcode_str(&payload.gcode, input)
        .map_err(ApiError::from_engine)?;

    Ok(Json(output))
}

async fn quote_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<QuoteOutput>, ApiError> {
    let mut form = FormOptions::default();
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(ApiError::from_multipart)?
    {
        let name = field
            .name()
            .ok_or_else(|| ApiError::BadRequest("multipart field missing name".into()))?
            .to_string();

        match name.as_str() {
            "file" => {
                if file_bytes.is_some() {
                    return Err(ApiError::BadRequest("multiple file fields provided".into()));
                }
                let filename = field.file_name().map(|s| s.to_string());
                form.filename = filename;
                let data = read_field_to_vec(&state, field).await?;
                file_bytes = Some(data);
            }
            other => {
                let value = field
                    .text()
                    .await
                    .map_err(|err| ApiError::BadRequest(format!("invalid field {other}: {err}")))?;
                form.apply(other, &value)?;
            }
        }
    }

    let data = file_bytes.ok_or_else(|| ApiError::BadRequest("missing file field".into()))?;
    let file_name = form
        .filename
        .clone()
        .unwrap_or_else(|| "upload".to_string());

    let material_id = form
        .material_id
        .clone()
        .unwrap_or_else(|| DEFAULT_MATERIAL_ID.to_string());

    let mut input = QuoteInput::material(material_id);
    input.infill_percent = Some(DEFAULT_INFILL_PERCENT);
    input.supports = true;
    input.cost_per_kg_override = None;
    input.base_fee_override = None;

    let kind = detect_model_kind(Some(&file_name), &data)?;
    let quote = match kind {
        ModelKind::Gcode => state
            .engine
            .quote_from_gcode_bytes(&data, input)
            .map_err(ApiError::from_engine)?,
        ModelKind::ThreeMf => state
            .engine
            .quote_from_3mf_bytes(&data, input)
            .map_err(ApiError::from_engine)?,
        ModelKind::Stl => state
            .engine
            .estimate_from_stl_bytes(&data, input)
            .map_err(ApiError::from_engine)?,
    };

    Ok(Json(quote))
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    max_upload_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
struct MaterialDTO {
    id: String,
    display_name: String,
    kind: String,
    density_g_cm3: f32,
    cost_per_kg: f32,
}

impl From<price_engine::MaterialProfile> for MaterialDTO {
    fn from(profile: price_engine::MaterialProfile) -> Self {
        Self {
            id: profile.id,
            display_name: profile.display_name,
            kind: match profile.kind {
                price_engine::material::MaterialKind::Fdm => "fdm".to_string(),
                price_engine::material::MaterialKind::Resin => "resin".to_string(),
            },
            density_g_cm3: profile.density_g_cm3,
            cost_per_kg: profile.cost_per_kg,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GcodeQuoteRequest {
    material_id: String,
    gcode: String,
}

#[derive(Default)]
struct FormOptions {
    filename: Option<String>,
    material_id: Option<String>,
}

impl FormOptions {
    fn apply(&mut self, field: &str, raw: &str) -> Result<(), ApiError> {
        let value = raw.trim();
        if value.is_empty() {
            return Ok(());
        }

        match field {
            "material_id" => self.material_id = Some(value.to_string()),
            other => {
                return Err(ApiError::BadRequest(format!("unknown form field: {other}")));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("processing error: {0}")]
    Engine(#[from] EngineError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage error: {0}")]
    Storage(#[from] async_tempfile::Error),
}

impl ApiError {
    fn from_engine(err: EngineError) -> Self {
        Self::Engine(err)
    }

    fn from_multipart(err: MultipartError) -> Self {
        error!(error = ?err, "failed to parse multipart");
        Self::BadRequest(format!("multipart error: {err}"))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, Json(ErrorBody { message })).into_response()
            }
            ApiError::Engine(err) => match &err {
                EngineError::InvalidInput(_) | EngineError::UnsupportedFormat(_) => {
                    (StatusCode::BAD_REQUEST, Json(ErrorBody::from(&err))).into_response()
                }
                EngineError::NotImplemented { .. } => {
                    (StatusCode::NOT_IMPLEMENTED, Json(ErrorBody::from(&err))).into_response()
                }
                other => {
                    error!(error = ?other, "internal engine error");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorBody::from(&err)),
                    )
                        .into_response()
                }
            },
            ApiError::Io(err) => {
                error!(error = ?err, "io error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        message: "temporary storage error".to_string(),
                    }),
                )
                    .into_response()
            }
            ApiError::Storage(err) => {
                error!(error = ?err, "storage error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        message: "temporary storage error".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

impl From<&EngineError> for ErrorBody {
    fn from(err: &EngineError) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

async fn read_field_to_vec(
    state: &AppState,
    mut field: axum::extract::multipart::Field<'_>,
) -> Result<Vec<u8>, ApiError> {
    let mut temp_file = TempFile::new_in(state.upload_dir.as_ref().clone()).await?;

    let mut data = Vec::new();
    let mut total: usize = 0;

    while let Some(chunk) = field.chunk().await.map_err(ApiError::from_multipart)? {
        total += chunk.len();
        if total > state.max_upload_bytes {
            return Err(ApiError::BadRequest(format!(
                "file exceeds {} bytes limit",
                state.max_upload_bytes
            )));
        }
        temp_file.write_all(&chunk).await.map_err(ApiError::Io)?;
        data.extend_from_slice(&chunk);
    }

    Ok(data)
}

fn detect_model_kind(file_name: Option<&str>, data: &[u8]) -> Result<ModelKind, ApiError> {
    if let Some(name) = file_name {
        if let Some(ext) = Path::new(name)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
        {
            return match ext.as_str() {
                "gcode" => Ok(ModelKind::Gcode),
                "3mf" => Ok(ModelKind::ThreeMf),
                "stl" => Ok(ModelKind::Stl),
                other => Err(ApiError::BadRequest(format!(
                    "unsupported file extension: .{other}"
                ))),
            };
        }
    }

    if data.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        return Ok(ModelKind::ThreeMf);
    }
    if data.starts_with(b"solid ") || data.starts_with(b"SOLID ") {
        return Ok(ModelKind::Stl);
    }
    if data.starts_with(b";") || data.starts_with(b"G") {
        return Ok(ModelKind::Gcode);
    }

    Err(ApiError::BadRequest("unable to detect file type".into()))
}
