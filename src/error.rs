use async_graphql::{ Error as GraphQLError, ErrorExtensions };
use axum::{ http::StatusCode, response::{ IntoResponse, Response } };
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // Database related errors
    DatabaseError(String),

    // Auth errors
    Unauthorized(String),
    Forbidden(String),

    // Validation errors
    ValidationError(String),

    // Not found errors
    NotFound(String),

    // External service errors
    ExternalServiceError(String),

    // Generic errors
    InternalServerError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            Self::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// Convert AppError to GraphQLError for GraphQL responses
impl AppError {
    pub fn to_graphql_error(&self) -> GraphQLError {
        match self {
            AppError::ValidationError(msg) => {
                GraphQLError::new(msg.clone()).extend_with(|_, e| {
                    e.set("code", "VALIDATION_ERROR");
                    e.set("status", 400);
                })
            }
            AppError::NotFound(msg) => {
                GraphQLError::new(msg.clone()).extend_with(|_, e| {
                    e.set("code", "NOT_FOUND");
                    e.set("status", 404);
                })
            }
            AppError::Unauthorized(msg) => {
                GraphQLError::new(msg.clone()).extend_with(|_, e| {
                    e.set("code", "UNAUTHORIZED");
                    e.set("status", 401);
                })
            }
            AppError::Forbidden(msg) => {
                GraphQLError::new(msg.clone()).extend_with(|_, e| {
                    e.set("code", "FORBIDDEN");
                    e.set("status", 403);
                })
            }
            | AppError::DatabaseError(msg)
            | AppError::ExternalServiceError(msg)
            | AppError::InternalServerError(msg) => {
                GraphQLError::new(msg.clone()).extend_with(|_, e| {
                    e.set("code", "INTERNAL_SERVER_ERROR");
                    e.set("status", 500);
                })
            }
        }
    }
}

// Convert AppError to Axum Response for REST endpoints or middleware
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            Self::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::ExternalServiceError(msg) => (StatusCode::BAD_GATEWAY, msg),
            Self::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        // You could return JSON here instead of plain text if preferred
        (status, message).into_response()
    }
}

// Convenience type for results in your application
pub type AppResult<T> = Result<T, AppError>;
