use serde::{Deserialize, Serialize};

use crate::{material::MaterialProfile, model::ModelMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteInput {
    pub material_id: String,
    pub infill_percent: Option<u8>,
    pub supports: bool,
    pub cost_per_kg_override: Option<f64>,
    pub base_fee_override: Option<f64>,
    pub quantity: u32,
    pub turnaround: Turnaround,
}

impl QuoteInput {
    pub fn material(material_id: impl Into<String>) -> Self {
        Self {
            material_id: material_id.into(),
            infill_percent: None,
            supports: true,
            cost_per_kg_override: None,
            base_fee_override: None,
            quantity: 1,
            turnaround: Turnaround::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuoteBreakdown {
    pub currency: String,
    pub material_cost: f64,
    pub labor_cost: f64,
    pub base_fee: f64,
    pub margin_multiplier: f64,
    pub quantity: u32,
    pub unit_total: f64,
    pub subtotal: f64,
    pub express_multiplier: f64,
    pub turnaround: Turnaround,
    pub discount_rate: f64,
    pub discount_amount: f64,
    pub review_required: bool,
    pub total: f64,
    pub is_estimate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteOutput {
    pub metadata: ModelMetadata,
    pub material: MaterialProfile,
    pub breakdown: QuoteBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Turnaround {
    #[default]
    Standard,
    Express,
}
