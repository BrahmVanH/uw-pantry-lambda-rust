use std::{ env, time::{ SystemTime, UNIX_EPOCH } };

use serde::{ Deserialize, Serialize };
use jsonwebtoken::{ decode, encode, DecodingKey, EncodingKey, Header, Validation };

use crate::error::AppError;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user ID
    pub email: String,
    pub exp: usize,
}

// Create jwt from user id and email
pub fn create_token(user_id: &str, email: &str) -> Result<String, AppError> {
    // Load secret from ENV
    let jwt_secret = env::var("JWT_SECRET").map_err(|e| AppError::EnvError(e))?;
    let secret_as_bytes = jwt_secret.as_bytes();

    let expiration =
        (
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?
                .as_secs() as usize
        ) +
        24 * 3600;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_as_bytes)).map_err(|e|
        AppError::Unauthorized(e.to_string())
    )
}

// Validate token against jwt secret
pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    // Load secret from ENV
    let jwt_secret = env::var("JWT_SECRET").map_err(|e| AppError::EnvError(e))?;
    let secret_as_bytes = jwt_secret.as_bytes();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_as_bytes),
        &Validation::default()
    ).map_err(|e| AppError::Unauthorized(e.to_string()))?;

    Ok(token_data.claims)
}
