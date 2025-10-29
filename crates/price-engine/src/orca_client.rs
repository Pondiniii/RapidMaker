use serde::{Deserialize, Serialize};
use crate::error::EngineError;
use crate::model::{FilamentStats, ModelKind, ModelMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceRequest {
    pub infill: Option<u8>,
    pub supports: bool,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceMetadata {
    pub filament_used_g: Option<f64>,
    pub filament_used_mm: Option<f64>,
    pub filament_cost: Option<f64>,
    pub print_time_seconds: Option<u32>,
    pub print_time_human: Option<String>,
    pub total_layers: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceResponse {
    pub success: bool,
    pub gcode: Option<String>,
    pub metadata: SliceMetadata,
    pub error: Option<String>,
}

/// Client for OrcaSlicer HTTP Worker
pub struct OrcaClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl OrcaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Slice a 3D model file and extract metadata
    pub async fn slice_file(
        &self,
        file_data: Vec<u8>,
        filename: String,
        request: SliceRequest,
    ) -> Result<ModelMetadata, EngineError> {
        let url = format!("{}/api/slice", self.base_url);

        // Build multipart request
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_data)
                    .file_name(filename)
            )
            .text("config", serde_json::to_string(&request).unwrap_or_default());

        let response = self
            .http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| EngineError::InvalidInput(format!("OrcaClient request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| EngineError::InvalidInput(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(EngineError::InvalidInput(format!(
                "OrcaClient error: {} - {}",
                status, body
            )));
        }

        let slice_response: SliceResponse = serde_json::from_str(&body)
            .map_err(|e| EngineError::InvalidInput(format!("Invalid response format: {}", e)))?;

        if !slice_response.success {
            return Err(EngineError::InvalidInput(
                slice_response
                    .error
                    .unwrap_or_else(|| "Unknown slicing error".to_string()),
            ));
        }

        // Convert SliceMetadata to ModelMetadata
        let metadata = ModelMetadata {
            kind: ModelKind::Gcode,
            geometry: Default::default(),
            filament: FilamentStats {
                filament_used_g: slice_response.metadata.filament_used_g,
                filament_used_mm: slice_response.metadata.filament_used_mm,
                print_time_human: slice_response.metadata.print_time_human,
                print_time_seconds: slice_response.metadata.print_time_seconds,
                filament_type: None,
                infill_percent: request.infill,
                supports_enabled: Some(request.supports),
                solid_ratio_percent: None,
            },
        };

        Ok(metadata)
    }
}
