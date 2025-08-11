use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use serde_json::json;
use actix_session::Session;

use crate::{
    database::Database,
    errors::AppError,
    models::{CreateFarmRequest, CreateFarmerRequest, FarmResponse, Farmer, FarmerLogin, FarmerSession, SendOtpRequest, VerifyPhoneRequest},
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
}

pub async fn verify_phone(
    db: web::Data<Database>,
    payload: web::Json<VerifyPhoneRequest>,
    session : Session
) -> ActixResult<HttpResponse> {

    let phone_number = payload.clone().phone_number;
    
    match services::farmer_service::verify_phone_number(&db, payload.into_inner()).await {
        Ok(success) => {
            if success == true {
                log::info!("Farmer verification completed");

                // get the farmer and the farm from the id . The farmer is gotten from the phone number

                let farmer : Farmer = sqlx::query_as::<_, Farmer>(
                    r#"
                        SELECT id , email , phone_number , first_name FROM farmers WHERE phone_number = $1
                        Limit 1
                    "#
                )
                .bind(&phone_number)
                .fetch_one(&db.pool)
                .await
                .map_err(|e| {
                log::error!("Failed to fetch farmer: {:?}", e);
                actix_web::error::ErrorInternalServerError("Error fetching farmer")
            })?;

                let farm : FarmResponse = sqlx::query_as::<_, FarmResponse>(
                    r#"
                        SELECT id  FROM farm WHERE phone_number = $1
                        Limit 1
                    "#
                )
                .bind(&phone_number)
                .fetch_one(&db.pool)
                .await
                .map_err(|e| {
                log::error!("Failed to fetch farm: {:?}", e);
                actix_web::error::ErrorInternalServerError("Error fetching farm")
            })?;
                
                let farmer = FarmerSession{
                    farmer_id : farmer.id , 
                    farm_id : farm.id,
                    name : farmer.first_name
                };

                let json = serde_json::to_string(&farmer).unwrap();
               session.insert("farmer", json).unwrap();
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

// Name would be changed . This is the point we send the otp. It is really after user types otp that we do the actual login

pub async fn farmer_login (
    db: web::Data<Database>,
    payload: web::Json<FarmerLogin>
) -> ActixResult<HttpResponse>{
    
    log::info!("Logging in farmer");

    match services::farmer_service::send_login_otp(&db, payload.into_inner()).await{
        Ok(_) =>{
            log::info!("Otp sent");
            return Ok(HttpResponse::Ok().json(
                json!(
                    {
                        "success": true 
                    }
                )
            ));
        },
        Err (AppError::ValidationError(msg)) => {
            log::info!("Validation Error");
            return Ok(HttpResponse::BadRequest().json(
                json!(
                    {
                        "success": false
                    }
                )
            ));
        }
        Err(_) => {
            log::info!("Server error");
            return Ok(HttpResponse::InternalServerError().json(
                json!(
                    {
                        "error" : "Server Error", 
                        "message" : "Server temporarily unavailable"
                    }
                )
            ));
        }
    }
    
}

pub async fn dashboard(session : Session) -> impl Responder{
    if let Some(sesh) = session.get::<String>("farmer").unwrap(){
        let farmer : FarmerSession = serde_json::from_str(&sesh).unwrap();
        HttpResponse::Ok().body(format!("Welcome back {}" , farmer.name))
    }
    else{
        HttpResponse::Unauthorized().body("Not logged in")
    }
}

pub async fn logout(session : Session) -> impl Responder{
    session.purge();
    HttpResponse::Ok().body("Logged out")
}

