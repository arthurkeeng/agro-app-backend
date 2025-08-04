use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;

use crate::{
    database::Database,
    errors::AppError,
    models::{CreateFarmRequest, CreateFarmerRequest, SendOtpRequest, VerifyPhoneRequest},
    services,
};

pub async fn register_farmer(
    db: web::Data<Database>,
    payload: web::Json<CreateFarmerRequest>,
) -> ActixResult<HttpResponse> {
    log::info!("Registering farmer with payload : {:?}", payload);

    match services::farmer_service::create_farmer(&db, payload.into_inner()).await {
        Ok(farmer) => {
            log::info!("Registered successfully");
            return Ok(HttpResponse::Created().json(farmer));
        }
        Err(AppError::ValidationError(msg)) => {
            log::warn!("Validation Error occured during registration :{}", msg);
            return Ok(HttpResponse::BadRequest().json(json!(
            {"error" : msg , "code" : "VALIDATION_ERROR"}
            )));
        }
        Err(err) => {
            log::error!("Error registering farmer {:?}", err);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error" : "Registration failed" , "code" : "INTERNAL_ERROR"
            })));
        }
    }
    todo!()
}

pub async fn verify_phone(
    db: web::Data<Database>,
    payload: web::Json<VerifyPhoneRequest>,
) -> ActixResult<HttpResponse> {
    log::info!("Verifying phone numer");

    match services::farmer_service::verify_phone_number(&db, payload.into_inner()).await {
        Ok(success) => {
            if success == true {
                log::info!("Farmer verification completed");
                Ok(HttpResponse::Ok().json(json!(
                    {
                        "success" : true , "message" : "Phone verified successfully"
                    }
                )))
            } else {
                log::warn!("Farmer verification failed");
                Ok(HttpResponse::BadRequest().json(json!(
                    {
                        "success" : false , "message" : "Phone verified failed"
                    }
                )))
            }
        }
        Err(e) => {
            log::error!("Server Error {:?}", e);
            Ok(HttpResponse::InternalServerError().json(json!(
                {

                        "error" : false , "code" : "Server Error"

                }
            )))
        }
    }
}

pub async fn resend_otp(
    db: &Database,
    payload: web::Json<SendOtpRequest>,
) -> ActixResult<HttpResponse> {
    log::info!("Resending otp ");
    match services::farmer_service::send_otp(db, &payload.phone_number).await {
        Ok(_) => {
            log::info!("Otp sent successfully");
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "OTP sent successfully"
            })))
        }
        Err(_e) => {
            log::info!("Otp sent successfully");
             Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to send OTP",
                "code": "INTERNAL_ERROR"
            })))
        }
    }
}
