use async_graphql::{ Error as GraphQLError, ErrorExtensions };
// use aws_sdk_dynamodb::error::SdkError;
use axum::{ http::StatusCode, response::{ IntoResponse, Response } };
use std::env::VarError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // Env errors
    #[error("Environment variable error: {0}")] EnvError(#[from] VarError),

    // Database related errors
    #[error("Database error: {0}")] DatabaseError(String),


    // Auth errors
    #[error("Unauthorized: {0}")] Unauthorized(String),

    #[error("Forbidden: {0}")] Forbidden(String),

    // Validation errors
    #[error("Validation error: {0}")] ValidationError(String),

    // Not found errors
    #[error("Not found: {0}")] NotFound(String),

    // External service errors
    #[error("External service error: {0}")] ExternalServiceError(String),

    // Generic errors
    #[error("Internal server error: {0}")] InternalServerError(String),
}

impl AppError {
    pub fn to_graphql_error(&self) -> GraphQLError {
        match self {
            AppError::EnvError(msg) => {
                GraphQLError::new(msg.clone().to_string()).extend_with(|_, e| {
                    e.set("code", "ENV_ERROR");
                    e.set("status", 404);
                })
            }
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
            Self::EnvError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
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
