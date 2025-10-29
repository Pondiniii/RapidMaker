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

struct StlEstimate {
    metadata: ModelMetadata,
    solid_volume_cm3: f64,
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

    pub fn quote_from_3mf_bytes(
        &self,
        data: &[u8],
        input: QuoteInput,
    ) -> Result<QuoteOutput, EngineError> {
        let metadata = parse_3mf_metadata(data)?;
        self.compute_quote(metadata, input, false)
    }

    pub fn estimate_from_stl_bytes(
        &self,
        data: &[u8],
        input: QuoteInput,
    ) -> Result<QuoteOutput, EngineError> {
        let estimate = estimate_stl_metadata(data, &input)?;
        let material = self.material(&input.material_id).ok_or_else(|| {
            EngineError::InvalidInput(format!("unknown material: {}", input.material_id))
        })?;

        let mut metadata = estimate.metadata;
        let density = material.density_g_cm3 as f64;
        let grams = estimate.solid_volume_cm3 * density;
        metadata.filament.filament_used_g = Some(grams);

        let filament_area_mm2 = PI * (FILAMENT_DIAMETER_MM / 2.0).powi(2);
        let length_mm = (estimate.solid_volume_cm3 * 1000.0) / filament_area_mm2;
        metadata.filament.filament_used_mm = Some(length_mm);
        metadata.filament.filament_type = Some(material.display_name.clone());

        self.compute_quote(metadata, input, true)
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

fn parse_3mf_metadata(data: &[u8]) -> Result<ModelMetadata, EngineError> {
    let mut archive = ZipArchive::new(Cursor::new(data))?;

    let mut metadata = ModelMetadata {
        kind: ModelKind::ThreeMf,
        ..Default::default()
    };

    if let Ok(mut file) = archive.by_name("Metadata/slice_info.config") {
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                    if e.name().as_ref() == b"filament" {
                        let mut attrs = HashMap::new();
                        for attr in e.attributes().flatten() {
                            if let Ok(value) = attr.decode_and_unescape_value(reader.decoder()) {
                                attrs.insert(attr.key.as_ref().to_vec(), value.to_string());
                            }
                        }

                        if let Some(value) = attrs.get(b"used_g" as &[u8]) {
                            if let Ok(parsed) = value.parse::<f64>() {
                                metadata.filament.filament_used_g = Some(parsed);
                            }
                        }

                        if let Some(value) = attrs.get(b"used_m" as &[u8]) {
                            if let Ok(parsed) = value.parse::<f64>() {
                                metadata.filament.filament_used_mm = Some(parsed * 1000.0);
                            }
                        }

                        if let Some(value) = attrs.get(b"type" as &[u8]) {
                            metadata.filament.filament_type = Some(value.clone());
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => {
                    return Err(EngineError::Parse(format!(
                        "failed to parse slice_info.config: {err}"
                    )))
                }
                _ => {}
            }
            buf.clear();
        }
    }

    // fall back to G-code embedded inside archive
    if metadata.filament.filament_used_g.is_none() {
        if let Some(gcode) = extract_first_gcode(&mut archive)? {
            let gcode_meta = parse_gcode_metadata(&gcode);
            merge_filament_stats(&mut metadata.filament, &gcode_meta.filament);
        }
    } else if metadata.filament.print_time_seconds.is_none() {
        if let Some(gcode) = extract_first_gcode(&mut archive)? {
            let gcode_meta = parse_gcode_metadata(&gcode);
            if metadata.filament.print_time_seconds.is_none() {
                metadata.filament.print_time_seconds = gcode_meta.filament.print_time_seconds;
            }
            if metadata.filament.print_time_human.is_none() {
                metadata.filament.print_time_human = gcode_meta.filament.print_time_human;
            }
        }
    }

    Ok(metadata)
}

fn extract_first_gcode(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Option<String>, EngineError> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with(".gcode") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            return Ok(Some(contents));
        }
    }
    Ok(None)
}

fn merge_filament_stats(target: &mut FilamentStats, source: &FilamentStats) {
    if target.filament_used_g.is_none() {
        target.filament_used_g = source.filament_used_g;
    }
    if target.filament_used_mm.is_none() {
        target.filament_used_mm = source.filament_used_mm;
    }
    if target.print_time_human.is_none() {
        target.print_time_human = source.print_time_human.clone();
    }
    if target.print_time_seconds.is_none() {
        target.print_time_seconds = source.print_time_seconds;
    }
    if target.filament_type.is_none() {
        target.filament_type = source.filament_type.clone();
    }
    if target.infill_percent.is_none() {
        target.infill_percent = source.infill_percent;
    }
    if target.supports_enabled.is_none() {
        target.supports_enabled = source.supports_enabled;
    }
    if target.solid_ratio_percent.is_none() {
        target.solid_ratio_percent = source.solid_ratio_percent;
    }
}

fn estimate_stl_metadata(data: &[u8], input: &QuoteInput) -> Result<StlEstimate, EngineError> {
    let mesh = stl_io::read_stl(&mut Cursor::new(data))
        .map_err(|err| EngineError::InvalidInput(format!("invalid STL: {err}")))?;

    if mesh.faces.is_empty() {
        return Err(EngineError::InvalidInput("STL has no triangles".into()));
    }

    let triangle_count = mesh.faces.len();

    // Compute bounding box and geometry stats
    let mut min_xyz = [f64::INFINITY; 3];
    let mut max_xyz = [f64::NEG_INFINITY; 3];
    let mut volume_acc = 0.0f64;
    let mut area_acc = 0.0f64;

    for face in &mesh.faces {
        let a = vec_of(mesh.vertices[face.vertices[0] as usize]);
        let b = vec_of(mesh.vertices[face.vertices[1] as usize]);
        let c = vec_of(mesh.vertices[face.vertices[2] as usize]);

        update_extents(&mut min_xyz, &mut max_xyz, &a);
        update_extents(&mut min_xyz, &mut max_xyz, &b);
        update_extents(&mut min_xyz, &mut max_xyz, &c);

        volume_acc += signed_tetra_volume(&a, &b, &c);
        area_acc += triangle_area(&a, &b, &c);
    }

    let volume_mm3 = volume_acc.abs();
    let volume_cm3 = volume_mm3 / 1000.0;
    let surface_area_mm2 = area_acc * 2.0; // double-sided surface similar to legacy estimator
    let surface_area_cm2 = surface_area_mm2 / 100.0;

    let bounds = [
        max_xyz[0] - min_xyz[0],
        max_xyz[1] - min_xyz[1],
        max_xyz[2] - min_xyz[2],
    ];

    let infill = input
        .infill_percent
        .unwrap_or(pricing::DEFAULT_INFILL_PERCENT)
        .clamp(MIN_INFILL_PERCENT, MAX_INFILL_PERCENT);

    let mut solid_ratio = estimate_solid_ratio(infill as f64);
    if input.supports {
        solid_ratio *= 1.15; // add ~15% for supports (heuristic)
    }
    solid_ratio = solid_ratio.clamp(0.0, 1.0);

    let solid_volume_cm3 = volume_cm3 * solid_ratio;
    // Mass and filament length are derived later when the material profile is known.

    let layer_count = (bounds[2].abs() / LAYER_HEIGHT_MM).max(1.0);
    let average_perimeter_mm = (bounds[0] + bounds[1]) * 2.0;
    let path_length = average_perimeter_mm * layer_count * solid_ratio;
    let estimated_seconds = (path_length / PRINT_SPEED_MM_PER_S).max(60.0);

    let mut metadata = ModelMetadata::default();
    metadata.kind = ModelKind::Stl;
    metadata.geometry.volume_cm3 = Some(volume_cm3);
    metadata.geometry.surface_area_cm2 = Some(surface_area_cm2);
    metadata.geometry.bounding_box_mm = Some(bounds);
    metadata.geometry.triangle_count = Some(triangle_count as u32);

    metadata.filament.infill_percent = Some(infill);
    metadata.filament.supports_enabled = Some(input.supports);
    metadata.filament.solid_ratio_percent = Some(solid_ratio * 100.0);
    metadata.filament.print_time_seconds = Some(estimated_seconds.round() as u32);
    metadata.filament.print_time_human = Some(format_duration(
        metadata.filament.print_time_seconds.unwrap(),
    ));

    Ok(StlEstimate {
        metadata,
        solid_volume_cm3,
    })
}

fn vec_of(vertex: stl_io::Vertex) -> [f64; 3] {
    [vertex[0] as f64, vertex[1] as f64, vertex[2] as f64]
}

fn update_extents(min_xyz: &mut [f64; 3], max_xyz: &mut [f64; 3], point: &[f64; 3]) {
    for i in 0..3 {
        if point[i].is_finite() {
            min_xyz[i] = min_xyz[i].min(point[i]);
            max_xyz[i] = max_xyz[i].max(point[i]);
        }
    }
}

fn signed_tetra_volume(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    let cross = cross(sub(*b, *a), sub(*c, *a));
    dot(*a, cross) / 6.0
}

fn triangle_area(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    0.5 * norm(cross(sub(*b, *a), sub(*c, *a)))
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn estimate_solid_ratio(infill_percent: f64) -> f64 {
    let infill_clamped = infill_percent.clamp(MIN_INFILL_PERCENT as f64, MAX_INFILL_PERCENT as f64);
    let perimeter_ratio = 0.20; // ~20% for 3 perimeters
    let top_bottom_ratio = 0.15; // ~15% for top/bottom skins
    let infill_ratio = (1.0 - perimeter_ratio - top_bottom_ratio) * (infill_clamped / 100.0);
    perimeter_ratio + top_bottom_ratio + infill_ratio
}

fn format_duration(seconds: u32) -> String {
    let mut remaining = seconds;
    let hours = remaining / 3600;
    remaining %= 3600;
    let minutes = remaining / 60;
    let secs = remaining % 60;

    let mut parts = SmallVec::<[String; 3]>::new();
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 && hours == 0 {
        parts.push(format!("{}s", secs));
    }

    if parts.is_empty() {
        "0s".to_string()
    } else {
        parts.join(" ")
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

    #[test]
    fn estimates_from_stl() {
        let engine = PriceEngine::new();
        const ASCII_STL: &str = r"solid tetrahedron
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 20 0 0
      vertex 0 20 0
    endloop
  endfacet
  facet normal 0 1 0
    outer loop
      vertex 0 0 0
      vertex 0 20 0
      vertex 0 0 20
    endloop
  endfacet
  facet normal 1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 0 20
      vertex 20 0 0
    endloop
  endfacet
  facet normal 1 1 1
    outer loop
      vertex 20 0 0
      vertex 0 0 20
      vertex 0 20 0
    endloop
  endfacet
endsolid tetrahedron
";

        let quote = engine
            .estimate_from_stl_bytes(ASCII_STL.as_bytes(), QuoteInput::material("pla"))
            .expect("stl estimate");

        assert_eq!(quote.metadata.kind, ModelKind::Stl);
        assert!(quote.metadata.geometry.volume_cm3.unwrap() > 0.0);
        assert!(quote.metadata.filament.filament_used_g.unwrap() > 0.0);
        assert!(quote.breakdown.is_estimate);
        assert!(quote.breakdown.total > 0.0);
    }
}
