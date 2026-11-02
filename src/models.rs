use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct MortgageInput {
    pub principal: f64,
    pub annual_rate: f64,
    pub years: u32,
    pub insurance_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct MortgageResult {
    pub monthly_payment: f64,
    pub total_payment: f64,
    pub total_interest: f64,
    pub total_insurance: f64,
    pub effective_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ComparisonRequest {
    pub mortgages: Vec<MortgageInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ComparisonResult {
    pub results: Vec<MortgageResult>,
    pub best_monthly: usize,
    pub best_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct SavedOffer {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub principal: f64,
    pub annual_rate: f64,
    pub years: u32,
    pub insurance_rate: Option<f64>,
    pub is_best_monthly: bool,
    pub is_best_total: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdateOfferNameRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateOfferRequest {
    pub project_id: String,
    pub name: String,
    pub principal: f64,
    pub annual_rate: f64,
    pub years: u32,
    pub insurance_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct SaveProjectOffersRequest {
    pub offers: Vec<SaveOfferRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct SaveOfferRequest {
    pub id: Option<String>,
    pub name: String,
    pub principal: f64,
    pub annual_rate: f64,
    pub years: u32,
    pub insurance_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CompareRequest {
    pub offers: Vec<MortgageInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CompareResponse {
    pub results: Vec<MortgageResult>,
    pub best_monthly: usize,
    pub best_total: usize,
}
