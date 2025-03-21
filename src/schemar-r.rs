use std::collections::HashMap;

use async_graphql::{ Context, Error, Object, SimpleObject, Result as GraphQLResult };
use aws_sdk_dynamodb::{ types::{ error::InternalServerError, AttributeValue }, Client };
use axum::http::response;
use serde::Serialize;
use tracing::{ debug, info, warn };
use crate::models::user::User;

use uuid::Uuid;

use crate::{ error::AppError, AppState, FailureResponse };
