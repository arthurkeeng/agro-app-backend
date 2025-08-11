use actix_web::{web, HttpResponse, Responder};
use sqlx::{Error as SqlxError};
use uuid::Uuid;

use crate::database::Database;
use crate::models::product::{NewProduct, Product, slugify};

#[derive(Debug, serde::Serialize)]
struct ApiError {
    error: String,
}


fn validate_new_product(p : &NewProduct ) -> Result<() , String>{
    if p.name.trim().is_empty(){
        return Err("Product name is required".into());
    }
    if p.category.trim().is_empty(){
        return Err("Category name is required".into());
    }
    if p.unit.trim().is_empty(){
        return Err("Unit name is required".into());
    }
    if p.price_cents < 0 {
        return Err("Price_cents must be >= 0".into());
    }
    if p.min_order_qty <= 0{
        return Err("Quantity available must be >= 0".into());
    }
    Ok(())
}

async fn insert_product(
    db : &Database , id : Uuid , payload : &NewProduct , slug: &str
) -> Result<Product , SqlxError>{

    let currency = payload.currency_code.clone().unwrap_or_else(|| "NGN".to_string());

    let status = payload.status.clone().unwrap_or_else(||"draft".to_string());
    let visibility = payload.visibility.clone().unwrap_or_else(||"both".to_string());

    let rec = sqlx::query_as::<_ , Product>(
        r#"
            INSERT INTO products (
            id , farmer_id , farm_id , name , slug , description , category , unit , tags , price_cents , currency_code , min_order_qty, quantity_available , organic , perishable , expected_harvest_date , expiry_date , status , visibility , images)
            VALUES (
            $1, $2, $3,
            $4, $5, $6,
            $7, $8, $9,
            $10, $11,
            $12, $13,
            $14, $15,
            $16, $17,
            $18, $19,
            $20
    )
    RETURNING 
        id, farmer_id, farm_id,
            name, slug, description,
            category, unit, tags,
            price_cents, currency_code,
            min_order_qty, quantity_available,
            organic, perishable,
            expected_harvest_date, expiry_date,
            status, visibility,
            images,
            created_at, updated_at
        "#
    )
    .bind(id)
    .bind(payload.farmer_id)
    .bind(payload.farm_id)
    .bind(&payload.name)
    .bind(slug)
    .bind(&payload.description)
    .bind(&payload.category)
    .bind(&payload.unit)
    .bind(&payload.tags)
    .bind(payload.price_cents)
    .bind(currency)
    .bind(payload.min_order_qty)
    .bind(payload.quantity_available)
    .bind(payload.organic)
    .bind(payload.perishable)
    .bind(payload.expected_harvest_date)
    .bind(payload.expiry_date)
    .bind(status)
    .bind(visibility)
    .bind(&payload.images)
    .fetch_one(&db.pool)
    .await?;

    Ok(rec)

}

pub async fn add_products(
    db : web::Data<Database>, 
    json : web::Json<NewProduct>
) -> impl Responder
{
    let payload = json.into_inner();

    if let Err(msg) = validate_new_product(&payload){
        return HttpResponse::BadRequest().json(ApiError{
            error : msg
        })
    }

    let base_slug = payload.slug.clone().unwrap_or_else(||
    slugify(&payload.name)
    );

    let mut slug = base_slug.clone();

    let id = Uuid::new_v4();

    // try insert 
    for attempt in 0..3{
        match insert_product(&db, id, &payload, &slug).await{
            Ok(product) => {
                return HttpResponse::Ok().json(product);
            }
            Err(SqlxError::Database(db_err)) => {
                // 23505 = unique_violation
                if db_err.code().as_deref() == Some("23505") && attempt < 2 {
                    let suffix = Uuid::new_v4().to_string();
                    let short = &suffix[suffix.len().saturating_sub(6)..];
                    slug = format!("{}-{}", base_slug, short);
                    continue;
                } else {
                    let detail = db_err.message().to_string();
                    return HttpResponse::Conflict().json(ApiError { error: format!("Database conflict: {}", detail) });
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(ApiError { error: format!("Failed to insert product: {}", e) });
            }
        }
    }

    HttpResponse::InternalServerError().json(
        ApiError{error : "Failed to create product after retries".into()}
    )
}