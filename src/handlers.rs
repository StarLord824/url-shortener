use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use redis::AsyncCommands;
use crate::{models::{ShortenRequest, StoredUrl, DestructionMode}, utils, error::ApiError};
use qrcode::QrCode;
use std::io::Cursor;
use validator::Validate;
use time::OffsetDateTime;

/// Handler for shortening URLs
pub async fn shorten_url(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<redis::aio::Connection>,
    payload: web::Json<ShortenRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate the request payload
    payload.validate()?;
    
    // Generate or use the custom alias
    let id = match &payload.custom_alias {
        Some(alias) if utils::exists_in_db(&pool, alias).await? => {
            return Err(ApiError::Conflict("Emoji combination already exists".into()))
        },
        Some(alias) => alias.clone(),
        None => utils::generate_emoji_id(&pool).await?,
    };

    // Create the stored URL object
    let stored_url = StoredUrl {
        id: id.clone(),
        original_url: payload.original_url.clone(),
        created_at: chrono::Utc::now(),
        expiration_time: None,
        click_count: None,
        destruction_mode: payload.destruction.clone(),
    };

    // Convert chrono::DateTime to time::OffsetDateTime
    let created_at = OffsetDateTime::from_unix_timestamp(stored_url.created_at.timestamp())
        .map_err(|_| ApiError::Internal("Invalid timestamp".into()))?;

    // Insert the URL into the database
    sqlx::query!(
        r#"INSERT INTO urls 
           (id, original_url, created_at, destruction_mode)
           VALUES ($1, $2, $3, $4)"#,
        stored_url.id,
        stored_url.original_url,
        created_at,
        serde_json::to_value(stored_url.destruction_mode)?
    )
    .execute(&**pool)
    .await?;

    // Cache the URL in Redis
    let mut conn = redis_conn.get_ref().clone();
    let _: () = conn.set_ex(&id, &stored_url.original_url, 3600).await?;

    // Return the shortened URL
    Ok(HttpResponse::Created().json(serde_json::json!({ "short_url": id })))
}

/// Handler for redirecting to the original URL
pub async fn redirect(
    pool: web::Data<PgPool>,
    redis_conn: web::Data<redis::aio::Connection>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let mut conn = redis_conn.get_ref().clone();
    
    // Check Redis cache first
    if let Ok(url) = redis::cmd("GET").arg(&id).query_async(&mut conn).await {
        return Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", url))
            .finish());
    }

    // Fallback to the database
    let mut url: StoredUrl = sqlx::query_as!(
        StoredUrl,
        r#"SELECT id, original_url, created_at, expiration_time, click_count, 
           destruction_mode as "destruction_mode: DestructionMode"
           FROM urls WHERE id = $1"#,
        id
    )
    .fetch_one(&**pool)
    .await?;

    // Handle destruction modes
    match &mut url.destruction_mode {
        DestructionMode::ClickFuse(remaining) => {
            *remaining -= 1;
            if *remaining <= 0 {
                utils::secure_delete_url(&pool, &id).await?;
            }
        }
        DestructionMode::TimeBomb(expiry) if std::time::SystemTime::now() > *expiry => {
            utils::secure_delete_url(&pool, &id).await?;
            return Err(ApiError::Gone);
        }
        _ => {}
    }

    // Redirect to the original URL
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", url.original_url))
        .finish())
}

/// Handler for generating QR codes
pub async fn qr_code(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let url = sqlx::query!(
        "SELECT original_url FROM urls WHERE id = $1",
        id
    )
    .fetch_one(&**pool)
    .await?;

    // Generate the QR code
    let code = QrCode::new(url.original_url.as_bytes())?;
    let image = code.render::<char>()
        .quiet_zone(false)
        .build();

    // Convert the image to bytes
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    image.write_to(&mut cursor, image::ImageOutputFormat::Png)?;

    // Return the image as a response
    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(bytes))
}

/// Handler for generating visual hashes
pub async fn visual_hash(
    path: web::Path<String>,
) -> HttpResponse {
    let id = path.into_inner();
    let image = utils::generate_visual_hash(&id);
    
    // Convert the image to bytes
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    image.write_to(&mut cursor, image::ImageOutputFormat::Png)
        .expect("Failed to write image");
    
    // Return the image as a response
    HttpResponse::Ok()
        .content_type("image/png")
        .body(bytes)
}