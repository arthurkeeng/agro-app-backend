use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Farmer {
    pub id: Uuid,
    pub phone_number: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub registration_channel: String,
    pub verification_status: String,
    pub profile_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFarmerRequest {
    pub phone_number: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub registration_channel: Option<String>,
    pub farm_data: Option<CreateFarmRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFarmRequest {
    pub farm_name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address_text: Option<String>,
    pub farm_size_hectares: Option<f64>,
    pub farm_type: Option<String>,
    pub primary_crops: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPhoneRequest {
    pub phone_number: String,
    pub otp_code: String,
}

#[derive(Debug, Serialize)]
pub struct FarmerResponse {
    pub id: Uuid,
    pub phone_number: String,
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub verification_status: String,
    pub profile_completed: bool,
}
