use std::collections::HashMap;

use async_graphql::{ Context, Object, Error };
use aws_sdk_dynamodb::{ types::AttributeValue, Client };
use tracing::{ info, warn };
use crate::models::user::User;

use crate::error::AppError;

// GraphQL Schema
//  Query root
#[derive(Debug)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn sup(&self) -> String {
        "sup, crabs?".to_string()
    }
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>, Error> {
        let table_name = "Users";
        // get db instance from context
        let db_client = ctx.data::<Client>().map_err(|e| {
            warn!("Failed to get db_client from context: {:?}", e);
            AppError::InternalServerError(
                "Failed to access application db_client".to_string()
            ).to_graphql_error()
        })?;

        // scan table for all users
        let response = db_client
            .scan()
            .table_name(table_name)
            .send().await
            .map_err(|e| {
                warn!("Failed to get db_client from context: {:?}", e);
                AppError::DatabaseError(
                    "Failed to get all users from db".to_string()
                ).to_graphql_error()
            })?;

        info!("get all users response: {:?}", response);

        let users = response
            .items()
            .iter()
            .filter_map(|item| User::from_item(item))
            .collect::<Vec<User>>();

        info!("users from response items: {:?}", users);

        Ok(users)
    }

    // Get user by ID
    async fn user_by_id(&self, ctx: &Context<'_>, user_id: String) -> Result<User, Error> {
        let table_name = "Users";

        // get db instance from context
        let db_client = ctx.data::<Client>().map_err(|e| {
            warn!("Failed to get db_client from context: {:?}", e);
            AppError::InternalServerError(
                "Failed to access application db_client".to_string()
            ).to_graphql_error()
        })?;

        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S(user_id.to_string()));

        let response = db_client
            .get_item()
            .table_name(table_name)
            .set_key(Some(key))
            .send().await
            .map_err(|e| {
                warn!("Failed to get db_client from context: {:?}", e);
                AppError::DatabaseError(
                    "Failed to get user by id from db".to_string()
                ).to_graphql_error()
            })?;

        // Check for Some item from db
        let item = response.item.ok_or_else(||
            AppError::DatabaseError("No user found with that ID".to_string())
        )?;

        // Return Some user converted from item or error
        User::from_item(&item).ok_or_else(||
            AppError::DatabaseError("No user found with that ID".to_string()).to_graphql_error()
        )
    }

    // Get user by email
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> Result<User, Error> {
        let table_name = "Users";
        let index_name = "EmailIndex";
        let key_condition_expression = "email = :email";

        // get db instance from context
        let db_client = ctx.data::<Client>().map_err(|e| {
            warn!("Failed to get db_client from context: {:?}", e);
            AppError::InternalServerError(
                "Failed to access application db_client".to_string()
            ).to_graphql_error()
        })?;

        let mut key = HashMap::new();
        key.insert("email".to_string(), AttributeValue::S(email.to_string()));

        let response = db_client
            .query()
            .table_name(table_name)
            .index_name(index_name)
            .key_condition_expression(key_condition_expression)
            .expression_attribute_values(":email", AttributeValue::S(email))
            .send().await
            .map_err(|e| {
                warn!("Failed to get db_client from context: {:?}", e);
                AppError::DatabaseError(
                    "Failed to get user by email from db".to_string()
                ).to_graphql_error()
            })?;
        let items = response.items();
        let first_item = items
            .first()
            .ok_or_else(||
                AppError::DatabaseError(
                    "No user found with that email address".to_string()
                ).to_graphql_error()
            )?;

        User::from_item(first_item).ok_or_else(||
            AppError::DatabaseError(
                "No user found with that email address".to_string()
            ).to_graphql_error()
        )
    }
}
