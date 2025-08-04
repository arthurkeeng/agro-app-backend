

use serde::{Deserialize , Serialize};
use uuid::Uuid;
use chrono::{NaiveDate , DateTime , Utc};
use Sqlx::types::Json;

#[derive(Debug , Serialize , Deserialize , sqlx::FromRow)]
pub struct FarmActivity {
    pub id : Uuid , 
    pub farmer_id : Uuid , 
    pub farm_id : Option<Uuid>,
    pub activity_type : String , 
    pub description : String , 
    pub activity_date : NaiveDate , 
    pub status : String , 
    pub crop_name : Option<String>, 
    pub field_plot : Option<String>, 
    pub inputs_used : Option<Json<Vec<InputUsed>>>,
    pub quantity_measured :Option<f64>, 
    pub unit_measured : Option<String>, 
    pub expected_harvest_date  : Option<NaiveDate>, 
    pub notes : Option<String>,
    pub created_at : DateTime<Utc> , 
    pub updated_at : DateTime<Utc>
}

#[derive (Debug , Serialize , Deserialize , Clone)]
pub struct InputUsed {
    pub item : String , 
    pub Quantity : f64 , 
    pub unit : String
}

#[derive(Debug , Deserialize)]
pub struct CreateFarmActivityRequest{
    pub farmer_id : Uuid , 
    pub farm_id : Option<Uuid>, 
    pub activity_type : String , 
    pub description : String , 
    pub activity_date : NaiveDate , 
    pub status : Option<String >,
    pub crop_name : Option <String> , 
    pub field_plot : Option<String>,
    pub inputs_used : Option<Vec<InputUsed>>,
    pub quantity_measured: Option<f64>,
    pub unit_measured: Option<String>,
    pub expected_harvest_date: Option<NaiveDate>,
    pub notes: Option<String>

}

#[derive(Debug, Deserialize)]
pub struct UpdateFarmActivityRequest {
    pub activity_type: Option<String>,
    pub description: Option<String>,
    pub activity_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub crop_name: Option<String>,
    pub field_plot: Option<String>,
    pub inputs_used: Option<Vec<InputUsed>>,
    pub quantity_measured: Option<f64>,
    pub unit_measured: Option<String>,
    pub expected_harvest_date: Option<NaiveDate>,
    pub notes: Option<String>,
}