use actix_web::App;
use sqlx::{Row, types::BigDecimal};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::{
    database::Database, 
    errors::{AppError, AppResult}, 
    models::{CreateFarmerRequest, FarmerResponse, FarmerLogin, LoginResponse, VerifyPhoneRequest}, 
    services::{self, sms_service}, 
    utils::generate_otp
};

pub async fn create_farmer(db: &Database, request: CreateFarmerRequest) -> AppResult<FarmerResponse> {
    if request.phone_number.is_empty() {
        return Err(AppError::ValidationError("Phone number cannot be empty".to_string()));
    }

    let existing_farmer = sqlx::query("SELECT id FROM farmers WHERE phone_number = $1")
        .bind(&request.phone_number)
        .fetch_optional(&db.pool)
        .await?;

    if existing_farmer.is_some() {
        return Err(AppError::ValidationError("Phone number already registered".to_string()));
    }

    let farmer_id = Uuid::new_v4();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO farmers (id, phone_number, email, first_name, last_name, registration_channel)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(farmer_id)
    .bind(&request.phone_number)
    .bind(&request.email)
    .bind(&request.first_name)
    .bind(&request.last_name)
    .bind(request.registration_channel.unwrap_or_else(|| "Web".to_string()))
    .execute(&mut *tx)
    .await?;

    if let Some(farm_data) = request.farm_data {
        let location = if let (Some(lat), Some(lng)) = (farm_data.latitude, farm_data.longitude) {
            Some(format!("POINT({} {})", lng, lat))
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO farms (farmer_id, farm_name, location, address_text, farm_size_hectares, farm_type, primary_crops)
            VALUES ($1, $2, ST_GeomFromText($3, 4326), $4, $5, $6, $7)
            "#
        )
        .bind(farmer_id)
        .bind(&farm_data.farm_name)
        .bind(location)
        .bind(&farm_data.address_text)
        .bind(BigDecimal::try_from(farm_data.farm_size_hectares.unwrap()).unwrap())
        .bind(farm_data.farm_type.unwrap_or_else(|| "subsistence".to_string()))
        .bind(serde_json::to_value(farm_data.primary_crops.unwrap_or_default()).unwrap())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    send_otp(&db, &request.phone_number).await?;

    Ok(FarmerResponse {
        id: farmer_id,
        phone_number: request.phone_number,
        email: request.email,
        first_name: request.first_name,
        last_name: request.last_name,
        verification_status: "Pending".to_string(),
        profile_completed: false,
    })
}



pub async fn send_otp(db: &Database, phone_number: &str) -> AppResult<()> {
    let otp_code = generate_otp();
    let expires_at = Utc::now() + Duration::minutes(30);

    sqlx::query(
        r#"
        DELETE FROM phone_verifications WHERE phone_number = $1 AND expires_at < NOW()
        "#,
    )
    .bind(phone_number)
    .execute(&db.pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO phone_verifications (phone_number, otp_code, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(phone_number)
    .bind(&otp_code)
    .bind(expires_at)
    .execute(&db.pool)
    .await?;

    let message = format!("Your otp code is {}", otp_code);
    services::sms_service::send_sms_twilio(phone_number, &message)
        .await
        .unwrap();

    Ok(())
}



pub async fn verify_phone_number(db: &Database, request: VerifyPhoneRequest) -> AppResult<bool> {
    let row = sqlx::query(
        r#"
        SELECT id, verified, attempts, expires_at 
        FROM phone_verifications 
        WHERE phone_number = $1 AND otp_code = $2 
        ORDER BY created_at DESC 
        LIMIT 1
        "#,
    )
    .bind(&request.phone_number)
    .bind(&request.otp_code)
    .fetch_optional(&db.pool)
    .await?;

    if let Some(row) = row {
        let id: Uuid = row.get("id");
        let verified: Option<bool> = row.get("verified");
        let attempts: Option<i32> = row.get("attempts");
        let expires_at: DateTime<Utc> = row.get("expires_at");

        if verified.unwrap_or(false) || attempts.unwrap_or(0) >= 3 || expires_at < Utc::now() {
            return Ok(false);
        }

        let mut tx = db.pool.begin().await?;

        sqlx::query("UPDATE phone_verifications SET verified = true WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE farmers SET verification_status = 'phone_verified' WHERE phone_number = $1")
            .bind(&request.phone_number)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        return Ok(true);
    }

    Ok(false)
}



pub async fn send_login_otp(db: &Database, request: FarmerLogin) -> AppResult<bool> {
    if request.phone_number.is_empty() {
        return Err(AppError::ValidationError("No phone number provided".to_string()));
    }

    let exists = sqlx::query("SELECT id FROM farmers WHERE phone_number = $1")
        .bind(&request.phone_number)
        .fetch_optional(&db.pool)
        .await?;

    if exists.is_none() {
        return Err(AppError::ValidationError("Farmer is not registered".to_string()));
    }

    send_otp(&db, &request.phone_number).await?;

    Ok(true)
}



pub async fn login_farmer_after_otp(db: &Database, request: VerifyPhoneRequest) -> AppResult<LoginResponse> {
    println!("logging after otp {:?}", request);
    match verify_phone_number(&db, request.clone()).await {
        Ok(true) => {
            let row = sqlx::query("SELECT id, phone_number, email, first_name, last_name FROM farmers WHERE phone_number = $1")
                .bind(&request.phone_number)
                .fetch_one(&db.pool)
                .await?;

            Ok(LoginResponse {
                id: row.get("id"),
                phone_number: row.get("phone_number"),
                email: row.get("email"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
            })
        }
        _ => {
            log::info!("Error logging in");
            Err(AppError::ValidationError("Otp Error".to_string()))
        }
    }
}
