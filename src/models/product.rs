use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Core product domain model (source of truth).
/// MVP: single price per product, stock at product level.
/// Later: extend with product_variants and product_images tables when needed.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub farmer_id: Uuid,
    pub farm_id: Option<Uuid>,

    pub name: String,
    pub slug: String,
    pub description: Option<String>,

    pub category: String,
    pub unit: String,
    pub tags: Vec<String>,

    /// Price stored in smallest currency unit (e.g., kobo) to avoid floating point issues.
    pub price_cents: i64,
    /// ISO-4217 currency code (e.g., "NGN", "USD")
    pub currency_code: String,

    pub min_order_qty: i32,
    pub quantity_available: i32,

    pub organic: bool,
    pub perishable: bool,

    pub expected_harvest_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,

    /// Publication status: "draft" | "published" | "archived"
    pub status: String,
    /// Visibility: "local_only" | "public" | "both"
    pub visibility: String,

    /// Images as array of URLs (text[] in DB)
    pub images: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload for creating a new product.
/// Server generates: id, timestamps; also computes slug if not provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProduct {
    pub farmer_id: Uuid,
    pub farm_id: Option<Uuid>,

    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,

    pub category: String,
    pub unit: String,
    pub tags: Vec<String>,

    pub price_cents: i64,
    pub currency_code: Option<String>,

    pub min_order_qty: i32,
    pub quantity_available: i32,

    pub organic: bool,
    pub perishable: bool,

    pub expected_harvest_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,

    pub status: Option<String>,     // default: "draft"
    pub visibility: Option<String>, // default: "both"

    pub images: Vec<String>,
}

impl Default for NewProduct {
    fn default() -> Self {
        Self {
            farmer_id: Uuid::nil(),
            farm_id: None,

            name: String::new(),
            slug: None,
            description: None,

            category: "uncategorized".to_string(),
            unit: "kg".to_string(),
            tags: vec![],

            price_cents: 0,
            currency_code: Some("NGN".to_string()),

            min_order_qty: 1,
            quantity_available: 0,

            organic: false,
            perishable: false,

            expected_harvest_date: None,
            expiry_date: None,

            status: Some("draft".to_string()),
            visibility: Some("both".to_string()),

            images: vec![],
        }
    }
}

/// Helper: basic slugify logic (server-side)
pub fn slugify(name: &str) -> String {
    let mut s = name.to_lowercase();
    // Replace non-alphanumeric with hyphens
    s = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    // Collapse multiple hyphens
    let mut collapsed = String::new();
    let mut prev_dash = false;
    for ch in s.chars() {
        if ch == '-' {
            if !prev_dash {
                collapsed.push(ch);
                prev_dash = true;
            }
        } else {
            collapsed.push(ch);
            prev_dash = false;
        }
    }
    collapsed.trim_matches('-').to_string()
}
