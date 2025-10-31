use std::collections::HashMap;
use std::f64::consts::PI;
use std::io::{Cursor, Read};

use crate::{
    error::EngineError,
    material::{MaterialCatalog, MaterialProfile},
    model::{FilamentStats, GeometryStats, ModelKind, ModelMetadata},
    pricing::{self, PricingInput},
    quote::{QuoteBreakdown, QuoteInput, QuoteOutput},
};
use quick_xml::events::Event;
use quick_xml::Reader;
use smallvec::SmallVec;
use zip::ZipArchive;

const MIN_INFILL_PERCENT: u8 = 0;
const MAX_INFILL_PERCENT: u8 = 100;
const PRINT_SPEED_MM_PER_S: f64 = 60.0; // heuristic average
const LAYER_HEIGHT_MM: f64 = 0.2;
const FILAMENT_DIAMETER_MM: f64 = 1.75;

pub struct PriceEngine {
    currency: String,
    materials: HashMap<String, MaterialProfile>,
}

impl Default for PriceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceEngine {
    pub fn new() -> Self {
        Self {
            currency: pricing::DEFAULT_CURRENCY.to_string(),
            materials: MaterialCatalog::with_defaults().into_inner(),
        }
    }

    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }

    pub fn install_material(mut self, profile: MaterialProfile) -> Self {
        self.materials.insert(profile.id.clone(), profile);
        self
    }

    pub fn material(&self, material_id: &str) -> Option<&MaterialProfile> {
        self.materials.get(material_id)
    }

    pub fn materials(&self) -> impl Iterator<Item = (&String, &MaterialProfile)> {
        self.materials.iter()
    }

    pub fn quote_from_gcode_bytes(
        &self,
        data: &[u8],
        input: QuoteInput,
    ) -> Result<QuoteOutput, EngineError> {
        let text = std::str::from_utf8(data)
            .map_err(|_| EngineError::InvalidInput("G-code file must be UTF-8".into()))?;
        self.quote_from_gcode_str(text, input)
    }

    pub fn quote_from_gcode_str(
        &self,
        gcode: &str,
        input: QuoteInput,
    ) -> Result<QuoteOutput, EngineError> {
        let mut metadata = parse_gcode_metadata(gcode);
        metadata.kind = ModelKind::Gcode;
        self.compute_quote(metadata, input, false)
    }

    /// Compute quote from ModelMetadata (already sliced via OrcaWorker)
    pub fn quote_from_metadata(
        &self,
        metadata: ModelMetadata,
        input: QuoteInput,
    ) -> Result<QuoteOutput, EngineError> {
        self.compute_quote(metadata, input, false)
    }

    fn compute_quote(
        &self,
        mut metadata: ModelMetadata,
        input: QuoteInput,
        is_estimate: bool,
    ) -> Result<QuoteOutput, EngineError> {
        let material = self.material(&input.material_id).ok_or_else(|| {
            EngineError::InvalidInput(format!("unknown material: {}", input.material_id))
        })?;

        if metadata.filament.filament_type.is_none() {
            metadata.filament.filament_type = Some(material.display_name.clone());
        }
        if metadata.filament.infill_percent.is_none() {
            metadata.filament.infill_percent = input
                .infill_percent
                .or(Some(pricing::DEFAULT_INFILL_PERCENT));
        }
        if metadata.filament.supports_enabled.is_none() {
            metadata.filament.supports_enabled = Some(input.supports);
        }

        let grams = metadata
            .filament
            .filament_used_g
            .ok_or_else(|| EngineError::InvalidInput("filament mass missing in metadata".into()))?;

        let cost_per_kg = input
            .cost_per_kg_override
            .unwrap_or(material.cost_per_kg as f64);
        let margin = material.margin_multiplier as f64;

        let base_fee = input.base_fee_override.unwrap_or_else(|| {
            if material.base_fee > 0.0 {
                material.base_fee as f64
            } else {
                pricing::BASE_FEE_PLN
            }
        });

        let material_cost_per_unit = grams / 1000.0 * cost_per_kg;
        let pricing = pricing::compute(PricingInput {
            quantity: input.quantity.max(1),
            turnaround: input.turnaround.clone(),
            base_fee,
            material_cost_per_unit,
            margin_multiplier: margin,
        });

        let breakdown = QuoteBreakdown {
            currency: self.currency.clone(),
            material_cost: pricing.material_cost_total - pricing.discount_amount,
            labor_cost: 0.0,
            base_fee: pricing.base_fee,
            margin_multiplier: margin,
            quantity: pricing.quantity,
            unit_total: pricing.unit_total,
            subtotal: pricing.subtotal_after_turnaround,
            express_multiplier: pricing.express_multiplier,
            turnaround: pricing.turnaround.clone(),
            discount_rate: pricing.discount_rate,
            discount_amount: pricing.discount_amount,
            review_required: pricing.review_required,
            total: pricing.total,
            is_estimate,
        };

        Ok(QuoteOutput {
            metadata,
            material: material.clone(),
            breakdown,
        })
    }
}

fn parse_number_after_equals(line: &str) -> Option<f64> {
    let (_, rhs) = line.split_once('=')?;
    rhs.trim()
        .split_whitespace()
        .next()
        .and_then(|token| token.replace(',', ".").parse::<f64>().ok())
}

fn parse_print_time_seconds(line: &str) -> Option<u32> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("estimated printing time") {
        return None;
    }

    let (_, rhs) = line.split_once('=')?;
    let tokens = rhs.trim();

    let mut total_seconds = 0u32;
    let mut current_number = String::new();

    for ch in tokens.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
            continue;
        }

        if current_number.is_empty() {
            continue;
        }

        let value: u32 = current_number.parse().ok()?;
        current_number.clear();

        match ch {
            'h' | 'H' => total_seconds += value * 3600,
            'm' | 'M' => total_seconds += value * 60,
            's' | 'S' => total_seconds += value,
            _ => {}
        }
    }

    if !current_number.is_empty() {
        if let Ok(value) = current_number.parse::<u32>() {
            total_seconds += value;
        }
    }

    if total_seconds == 0 {
        None
    } else {
        Some(total_seconds)
    }
}

fn parse_gcode_metadata(gcode: &str) -> ModelMetadata {
    let mut filament = FilamentStats::default();

    for line in gcode.lines().filter(|line| line.starts_with(';')) {
        let lower = line.to_ascii_lowercase();

        if lower.contains("filament used [g]") {
            if let Some(value) = parse_number_after_equals(line) {
                filament.filament_used_g = Some(value);
            }
        } else if lower.contains("filament used [mm]") {
            if let Some(value) = parse_number_after_equals(line) {
                filament.filament_used_mm = Some(value);
            }
        } else if lower.contains("filament_type") {
            filament.filament_type = line.split('=').nth(1).map(|s| s.trim().to_string());
        } else if lower.contains("estimated printing time") {
            filament.print_time_human = line.split('=').nth(1).map(|s| s.trim().to_string());
        }

        if let Some(seconds) = parse_print_time_seconds(line) {
            filament.print_time_seconds = Some(seconds);
        }
    }

    ModelMetadata {
        kind: ModelKind::Gcode,
        geometry: GeometryStats::default(),
        filament,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GCODE: &str = "; filament used [g] = 4.94\n; filament used [mm] = 1234.56\n; filament_type = PLA\n; estimated printing time (normal mode) = 15m 32s\nG28\n";

    #[test]
    fn parses_basic_gcode_metadata() {
        let meta = parse_gcode_metadata(SAMPLE_GCODE);
        assert_eq!(meta.filament.filament_used_g, Some(4.94));
        assert_eq!(meta.filament.filament_used_mm, Some(1234.56));
        assert_eq!(meta.filament.filament_type.as_deref(), Some("PLA"));
        assert_eq!(meta.filament.print_time_seconds, Some(15 * 60 + 32));
    }

    #[test]
    fn quote_requires_known_material() {
        let engine = PriceEngine::new();
        let result = engine.quote_from_gcode_str(SAMPLE_GCODE, QuoteInput::material("unknown"));
        assert!(matches!(result, Err(EngineError::InvalidInput(_))));
    }

    #[test]
    fn quote_happy_path_from_gcode() {
        let engine = PriceEngine::new();
        let output = engine
            .quote_from_gcode_str(SAMPLE_GCODE, QuoteInput::material("pla"))
            .expect("quote should work");

        assert_eq!(output.metadata.filament.filament_used_g, Some(4.94));
        assert!(!output.breakdown.is_estimate);
        assert!(output.breakdown.total > 0.0);
    }

    #[test]
    fn parses_3mf_metadata_with_gcode_fallback() {
        use std::io::Write;
        use zip::write::FileOptions;

        let mut buffer = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buffer);
            let options = FileOptions::default();
            zip.start_file("Metadata/slice_info.config", options)
                .unwrap();
            zip.write_all(
                b"<plate><filament type=\"ABS\" used_g=\"12.3\" used_m=\"4.56\"/></plate>",
            )
            .unwrap();
            zip.start_file("Metadata/plate_1.gcode", options).unwrap();
            zip.write_all(SAMPLE_GCODE.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        let engine = PriceEngine::new();
        let quote = engine
            .quote_from_3mf_bytes(buffer.get_ref(), QuoteInput::material("pla"))
            .expect("3mf quote");

        assert_eq!(quote.metadata.kind, ModelKind::ThreeMf);
        assert_eq!(quote.metadata.filament.filament_used_g, Some(12.3));
        assert_eq!(quote.metadata.filament.filament_used_mm, Some(4560.0));
        assert!(quote.metadata.filament.print_time_seconds.is_some());
        assert!(!quote.breakdown.is_estimate);
    }

}
