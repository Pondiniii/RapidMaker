use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    Fdm,
    Resin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProfile {
    pub id: String,
    pub display_name: String,
    pub kind: MaterialKind,
    pub density_g_cm3: f32,
    pub cost_per_kg: f32,
    pub base_fee: f32,
    pub margin_multiplier: f32,
}

impl MaterialProfile {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        kind: MaterialKind,
        density_g_cm3: f32,
        cost_per_kg: f32,
        base_fee: f32,
        margin_multiplier: f32,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            kind,
            density_g_cm3,
            cost_per_kg,
            base_fee,
            margin_multiplier,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MaterialCatalog {
    entries: HashMap<String, MaterialProfile>,
}

impl MaterialCatalog {
    pub fn with_defaults() -> Self {
        let mut entries = HashMap::new();

        // FDM materials (values derived from legacy Python calculator)
        entries.insert(
            "pla".into(),
            MaterialProfile::new("pla", "PLA", MaterialKind::Fdm, 1.25, 90.0, 0.0, 1.0),
        );
        entries.insert(
            "abs".into(),
            MaterialProfile::new("abs", "ABS", MaterialKind::Fdm, 1.05, 125.0, 0.0, 1.0),
        );
        entries.insert(
            "pc".into(),
            MaterialProfile::new("pc", "PC", MaterialKind::Fdm, 1.2, 240.0, 0.0, 1.0),
        );
        entries.insert(
            "tpu".into(),
            MaterialProfile::new("tpu", "TPU", MaterialKind::Fdm, 1.25, 210.0, 0.0, 1.0),
        );

        Self { entries }
    }

    pub fn into_inner(self) -> HashMap<String, MaterialProfile> {
        self.entries
    }

    pub fn get(&self, id: &str) -> Option<&MaterialProfile> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &MaterialProfile)> {
        self.entries.iter()
    }
}
