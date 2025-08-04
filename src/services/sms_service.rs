use crate::errors::{AppError, AppResult};



use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn send_sms_twilio(phone_number: &str, message: &str) -> AppResult<()> {
    let account_sid = env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| AppError::InternalError("TWILIO_ACCOUNT_SID not set".to_string()))?;
    let auth_token = env::var("TWILIO_AUTH_TOKEN")
        .map_err(|_| AppError::InternalError("TWILIO_AUTH_TOKEN not set".to_string()))?;
    let from_number = env::var("TWILIO_PHONE_NUMBER")
        .map_err(|_| AppError::InternalError("TWILIO_PHONE_NUMBER not set".to_string()))?;

    let client = Client::new();
    let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid);
    let phone_number = tweak_phone_number(phone_number).unwrap();
    let response = client
        .post(&url)
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[
            ("From", from_number.as_str()),
            ("To", &phone_number),
            ("Body", message),
        ])
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to send SMS: {}", e)))?;

    if response.status().is_success() {
        log::info!("SMS sent successfully to {}", phone_number);
        return Ok(());
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return  Err(AppError::InternalError(format!("SMS sending failed: {}", error_text)))
    }

    Ok(())
}

fn tweak_phone_number(phone_number: &str) -> Option<String>{
    if phone_number.starts_with("0") && phone_number.len() == 11{
        Some(format!("+234{}" , &phone_number[1..]))
    }
    else if phone_number.starts_with("+234") && phone_number.len() == 14 {
        Some(phone_number.to_string())
    }
    else{
        None
    }
}

