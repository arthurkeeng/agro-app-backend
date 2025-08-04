use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PhoneVerification {
    pub id: Uuid,
    pub phone_number: String,
    pub otp_code: String,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub phone_number: String,
}
