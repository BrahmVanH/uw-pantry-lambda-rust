use axum::{
    body::Body,
    http::{ header::AUTHORIZATION, HeaderMap, Request },
    middleware::Next,
    response::Response,
};

use crate::error::AppError;

use super::jwt::validate_token;

pub async fn auth_middleware<B>(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next
) -> Result<Response, AppError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("No authorization header".into()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid token format".into()));
    }

    let token = &auth_header[7..];

    let claims = validate_token(token)?;

    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
