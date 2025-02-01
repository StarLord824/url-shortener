use crate::{error::ApiError};
use sqlx::PgPool;
use sha2::{Sha256, Digest};
use image::{RgbaImage, Rgba};
use rand::Rng;

const EMOJI_RANGES: &[(u32, u32)] = &[
    (0x1F600, 0x1F64F),
    (0x1F300, 0x1F5FF),
    (0x1F680, 0x1F6FF),
    (0x1F900, 0x1F9FF),
    (0x2600, 0x26FF),
];

pub async fn exists_in_db(pool: &PgPool, id: &str) -> Result<bool, ApiError> {
    let exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM urls WHERE id = $1)",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    Ok(exists)
}


pub async fn generate_emoji_id(pool: &PgPool) -> Result<String, ApiError> {
    let mut rng = rand::thread_rng();
    loop {
        let id: String = (0..3)
            .map(|_| {
                let range = EMOJI_RANGES[rng.gen_range(0..EMOJI_RANGES.len())];
                char::from_u32(rng.gen_range(range.0..=range.1)).unwrap()
            })
            .collect();
        
        if !exists_in_db(pool, &id).await? {
            return Ok(id);
        }
    }
}

pub fn generate_visual_hash(id: &str) -> RgbaImage {
    let hash = Sha256::digest(id.as_bytes());
    let size = 256;
    let mut image = RgbaImage::new(size, size);

    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let hash_byte = hash[(x as usize + y as usize) % hash.len()];
        *pixel = Rgba([
            hash_byte.wrapping_add(x as u8),
            hash_byte.wrapping_sub(y as u8),
            hash_byte ^ (x as u8).wrapping_add(y as u8),
            255,
        ]);
    }
    image
}

pub async fn secure_delete_url(pool: &PgPool, id: &str) -> Result<(), ApiError> {
    let noise: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(256)
        .map(char::from)
        .collect();

    sqlx::query!(
        "UPDATE urls SET original_url = $1 WHERE id = $2",
        noise,
        id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "DELETE FROM urls WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}