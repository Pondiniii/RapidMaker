use crate::quote::Turnaround;

pub const DEFAULT_CURRENCY: &str = "PLN";
pub const DEFAULT_INFILL_PERCENT: u8 = 25;
pub const BASE_FEE_PLN: f64 = 50.0;
pub const EXPRESS_SURCHARGE_RATE: f64 = 0.35;
pub const AUTO_QUOTE_MAX_QTY: u32 = 30;
pub const DISCOUNT_THRESHOLD: u32 = 40;
pub const DISCOUNT_STEP: u32 = 10;
pub const DISCOUNT_PER_STEP: f64 = 0.05;
const MAX_DISCOUNT_RATE: f64 = 0.35;

#[derive(Debug, Clone)]
pub struct PricingInput {
    pub quantity: u32,
    pub turnaround: Turnaround,
    pub base_fee: f64,
    pub material_cost_per_unit: f64,
    pub margin_multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct PricingOutput {
    pub quantity: u32,
    pub turnaround: Turnaround,
    pub express_multiplier: f64,
    pub discount_rate: f64,
    pub discount_amount: f64,
    pub material_cost_total: f64,
    pub base_fee: f64,
    pub subtotal_before_margin: f64,
    pub subtotal_after_turnaround: f64,
    pub total: f64,
    pub unit_total: f64,
    pub review_required: bool,
}

pub fn compute(input: PricingInput) -> PricingOutput {
    let quantity = input.quantity.max(1);
    let discount_rate = discount_rate_for(quantity);
    let express_multiplier = express_multiplier(&input.turnaround);

    let material_cost_total = input.material_cost_per_unit * quantity as f64;
    let discountable = material_cost_total;
    let discount_amount = (discountable * discount_rate).min(discountable);

    let subtotal_before_margin = input.base_fee + material_cost_total - discount_amount;
    let subtotal_after_turnaround = subtotal_before_margin * express_multiplier;
    let total = subtotal_after_turnaround * input.margin_multiplier;
    let unit_total = total / quantity as f64;

    PricingOutput {
        quantity,
        turnaround: input.turnaround,
        express_multiplier,
        discount_rate,
        discount_amount,
        material_cost_total,
        base_fee: input.base_fee,
        subtotal_before_margin,
        subtotal_after_turnaround,
        total,
        unit_total,
        review_required: quantity > AUTO_QUOTE_MAX_QTY,
    }
}

pub fn express_multiplier(turnaround: &Turnaround) -> f64 {
    match turnaround {
        Turnaround::Express => 1.0 + EXPRESS_SURCHARGE_RATE,
        Turnaround::Standard => 1.0,
    }
}

pub fn discount_rate_for(quantity: u32) -> f64 {
    if quantity <= DISCOUNT_THRESHOLD {
        return 0.0;
    }

    let steps = ((quantity - DISCOUNT_THRESHOLD - 1) / DISCOUNT_STEP) + 1;
    let rate = steps as f64 * DISCOUNT_PER_STEP;
    rate.min(MAX_DISCOUNT_RATE)
}
