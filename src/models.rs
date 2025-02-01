use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use lazy_static::lazy_static;
use fancy_regex::Regex;
use sqlx::{Type, postgres::PgTypeInfo};

#[derive(Debug, Deserialize, Validate)]
pub struct ShortenRequest {
    #[validate(url)]
    pub original_url: String,
    
    #[validate(length(min = 3, max = 3), regex = "EMOJI_REGEX")]
    pub custom_alias: Option<String>,
    
    #[serde(default)]
    pub destruction: DestructionMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredUrl {
    pub id: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub expiration_time: Option<SystemTime>,
    pub click_count: Option<i32>,
    pub destruction_mode: DestructionMode,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum DestructionMode {
    #[default]
    Permanent,
    TimeBomb(SystemTime),
    ClickFuse(i32),
    Kombinatio(Box<[DestructionMode; 2]>),
}

impl Type<sqlx::Postgres> for DestructionMode {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for DestructionMode {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let json: serde_json::Value = sqlx::Decode::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DestructionMode {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        let json = serde_json::to_value(self).unwrap();
        json.encode_by_ref(buf)
    }
}

lazy_static! {
    static ref EMOJI_REGEX: Regex = 
        Regex::new(r"^(\p{Emoji}\p{Emoji_Modifier}?){3}$").unwrap();
}