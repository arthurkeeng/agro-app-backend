use actix_web::App;
use sqlx::types::BigDecimal;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::{
    database::Database, errors::{AppError, AppResult}, models::{CreateFarmerRequest, Farmer, FarmerResponse, VerifyPhoneRequest}, services::{self, sms_service}, utils::generate_otp
};

pub async fn create_farmer (db : &Database , request : CreateFarmerRequest) ->AppResult<FarmerResponse>{
    // validate farmer phone number
    
    if request.phone_number.is_empty(){
        return Err(AppError::ValidationError("Phone number cannot be empty".to_string()))
    }

    // check if farmer already exists

  
    let existing_farmer = sqlx::query!(
        "SELECT id FROM farmers WHERE phone_number = $1", 
        request.phone_number
    ).fetch_optional(&db.pool).await?;

    if existing_farmer.is_some(){
        return Err(AppError::ValidationError("Phone number already registered".to_string()));
    }

    let farmer_id = Uuid::new_v4();

    // start transaction

    let mut tx = db.pool.begin().await?;
    sqlx::query!(
        r#"
        INSERT INTO farmers (id, phone_number, email, first_name, last_name, registration_channel)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        farmer_id,
        request.phone_number,
        request.email,
        request.first_name,
        request.last_name,
        request.registration_channel.unwrap_or_else(||"Web".to_string())
    )
    .execute(&mut *tx)
    .await?;
    

    if let Some(farm_data) = request.farm_data {
        let location = if let (Some(lat), Some(lng)) = (farm_data.latitude, farm_data.longitude) {
            Some(format!("POINT({} {})", lng, lat))
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO farms (farmer_id, farm_name, location, address_text, farm_size_hectares, farm_type, primary_crops)
            VALUES ($1, $2, ST_GeomFromText($3, 4326), $4, $5, $6, $7)
            "#,
            farmer_id,
            farm_data.farm_name,
            location,
            farm_data.address_text,
            BigDecimal::try_from(farm_data.farm_size_hectares.unwrap()).unwrap(),
            farm_data.farm_type.unwrap_or_else(|| "subsistence".to_string()),
            serde_json::to_value(farm_data.primary_crops.unwrap_or_default()).unwrap()
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    // Send otp

    send_otp(&db, &request.phone_number).await?;


    Ok(FarmerResponse { id: farmer_id, phone_number: request.phone_number, email: request.email, first_name: request.first_name, last_name: request.last_name, verification_status: "Pending".to_string(), profile_completed: false })
}


pub async fn send_otp(db: &Database , phone_number : &str)-> AppResult<()>{
    let otp_code = generate_otp();
    let expires_at = Utc::now() + Duration::minutes(50);

    // remove old otp 
    sqlx::query!(
        r#"
            DELETE FROM phone_verifications WHERE phone_number = $1 AND expires_at < NOW()
        "#, phone_number
    )
    .execute(&db.pool).await?;

    // Store new otp in database

    
    sqlx::query!(
         r#"
        INSERT INTO phone_verifications (phone_number, otp_code, expires_at)
        VALUES ($1, $2, $3)
        "#,
        phone_number , otp_code , expires_at
    ).execute(&db.pool).await?;

    let message = format!("Your otp code is {}" , otp_code);
    services::sms_service::send_sms_twilio(phone_number, &message).await.unwrap();
    Ok(())
}


pub async fn verify_phone_number(db : &Database , request : VerifyPhoneRequest) -> AppResult<bool>{
    println!("Verifying phone");
    let verification = sqlx::query!(
        r#"
        SELECT id , verified , attempts , expires_at FROM phone_verifications
        WHERE phone_number =  $1 AND otp_code = $2 ORDER BY created_at DESC
        LIMIT 1
        "#,request.phone_number , request.otp_code 

    )
    .fetch_optional(&db.pool)
    .await?;

    if let Some(verification) = verification {
        if verification.verified.unwrap() {
            return Ok(false)
        }
        if verification.attempts.unwrap() >= 3 {
            return Ok(false)
        }
        if verification.expires_at < Utc::now(){
            return Ok(false)
        }

        // start transaction
        let mut tx = db.pool.begin().await?;

        sqlx::query!(
            r#"
            UPDATE phone_verifications SET verified = true WHERE id = $1
            "# , verification.id
        )
        .execute(&mut *tx)
        .await?;


        // update verification status
       sqlx::query!(
            "UPDATE farmers SET verification_status = 'phone_verified' WHERE phone_number = $1",
            request.phone_number
        )
        .execute(&mut *tx)
        .await?;

         tx.commit().await?;
         return Ok(true)
        
    }

    Ok(false)
}
