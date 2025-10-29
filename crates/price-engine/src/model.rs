use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Gcode,
    ThreeMf,
    Stl,
}

impl Default for ModelKind {
    fn default() -> Self {
        Self::Gcode
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeometryStats {
    pub volume_cm3: Option<f64>,
    pub surface_area_cm2: Option<f64>,
    pub bounding_box_mm: Option<[f64; 3]>,
    pub triangle_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilamentStats {
    pub filament_used_g: Option<f64>,
    pub filament_used_mm: Option<f64>,
    pub print_time_human: Option<String>,
    pub print_time_seconds: Option<u32>,
    pub filament_type: Option<String>,
    pub infill_percent: Option<u8>,
    pub supports_enabled: Option<bool>,
    pub solid_ratio_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub kind: ModelKind,
    pub geometry: GeometryStats,
    pub filament: FilamentStats,
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            kind: ModelKind::default(),
            geometry: GeometryStats::default(),
            filament: FilamentStats::default(),
        }
    }
}
