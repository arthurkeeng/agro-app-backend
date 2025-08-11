use serde::{Serialize , Deserialize};



#[derive(Debug, Serialize, Deserialize , sqlx::FromRow)]
pub struct FarmResponse {
    pub id : String
}