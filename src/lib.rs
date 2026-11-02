pub mod models;
pub use models::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "wasm")]
use web_sys::{Request, RequestInit, Response};

pub fn calculate_mortgage(input: &MortgageInput) -> MortgageResult {
    let monthly_rate = input.annual_rate / 100.0 / 12.0;
    let months = input.years as f64 * 12.0;

    let monthly_payment = if monthly_rate == 0.0 {
        input.principal / months
    } else {
        input.principal * monthly_rate * (1.0 + monthly_rate).powi(months as i32)
            / ((1.0 + monthly_rate).powi(months as i32) - 1.0)
    };

    let total_payment = monthly_payment * months;
    let total_interest = total_payment - input.principal;

    let insurance_rate = input.insurance_rate.unwrap_or(0.0) / 100.0;
    let monthly_insurance = input.principal * insurance_rate / 12.0;
    let total_insurance = monthly_insurance * months;

    let effective_rate = if input.principal > 0.0 {
        ((total_payment + total_insurance) / input.principal - 1.0) / input.years as f64 * 100.0
    } else {
        0.0
    };

    MortgageResult {
        monthly_payment: monthly_payment + monthly_insurance,
        total_payment: total_payment + total_insurance,
        total_interest,
        total_insurance,
        effective_rate,
    }
}

pub fn compare_mortgages(request: ComparisonRequest) -> ComparisonResult {
    let results: Vec<MortgageResult> = request.mortgages
        .iter()
        .map(calculate_mortgage)
        .collect();

    let best_monthly = results
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.monthly_payment.partial_cmp(&b.1.monthly_payment).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let best_total = results
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_payment.partial_cmp(&b.1.total_payment).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    ComparisonResult {
        results,
        best_monthly,
        best_total,
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn handle_request(url: &str, method: &str, body: Option<String>) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method(method);
    
    if let Some(body_str) = body {
        opts.set_body(&body_str.into());
    }
    
    let request = Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    
    let window = web_sys::window().expect("no global `window` exists");
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    
    let text = JsFuture::from(resp.text()?).await?;
    Ok(text)
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn calculate_single_mortgage_json(json: &str) -> Result<String, JsValue> {
    let input: MortgageInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;
    let result = calculate_mortgage(&input);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn compare_mortgages_json(json: &str) -> Result<String, JsValue> {
    let request: ComparisonRequest = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;
    let result = compare_mortgages(request);
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
