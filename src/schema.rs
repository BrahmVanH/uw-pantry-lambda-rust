use std::collections::HashMap;

use async_graphql::{ Context, Error, Object, SimpleObject, Result as GraphQLResult };
use aws_sdk_dynamodb::{ types::{ error::InternalServerError, AttributeValue }, Client };
use axum::http::response;
use serde::Serialize;
use tracing::{ debug, info, warn };
use crate::models::user::User;

use uuid::Uuid;

use crate::{ error::AppError, AppState, FailureResponse };

// GraphQL Schema
//  Query root
#[derive(Debug)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn sup(&self) -> String {
        "sup, crabs?".to_string()
    }
    async fn users(&self, ctx: &Context<'_>) -> GraphQLResult<Vec<User>> {
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
    async fn user_by_id(&self, ctx: &Context<'_>, user_id: String) -> GraphQLResult<Option<User>> {
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

        if let Some(item) = response.item {
            let user = User::from_item(&item);
            Ok(user)
        } else {
            Ok(None)
        }
    }

    // Get user by email
    async fn user_by_email(&self, ctx: &Context<'_>, email: String) -> GraphQLResult<Option<User>> {
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

        if let Some(user) = items.first() {
            Ok(User::from_item(user))
        } else {
            Ok(None)
        }
    }
}

// Mutation root
#[derive(Debug)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // Creates user in database
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
        pantry_name: String,
        first_name: String,
        last_name: String
    ) -> Result<User, Error> {
        // Transform context error into our AppError, then into GraphQL error
        info!("creating new user: {}", email);
        let db_client = ctx.data::<Client>().map_err(|e| {
            warn!("Failed to get db_client from context: {:?}", e);
            AppError::InternalServerError(
                "Failed to access application db_client".to_string()
            ).to_graphql_error()
        })?;

        info!("successfully created db_client: {:?}", &db_client);

        let id = Uuid::new_v4().to_string();

        // Generate User struct instance from params
        let user = match User::new(id, email, &password, first_name, last_name, pantry_name) {
            Ok(u) => u,
            Err(e) => {
                return Err(AppError::InternalServerError(e).to_graphql_error());
            }
        };

        // Turn User struct into DynamoDB Item
        let item = user.to_item();

        let put_item_output = db_client
            .put_item()
            .table_name("Users")
            .set_item(Some(item))
            .send().await
            .map_err(|err| {
                warn!("Database error while creating user: {}", err);
                AppError::DatabaseError(
                    format!("Failed to create user: {}", err)
                ).to_graphql_error()
            });
        info!("put_item_output: {:?}", &put_item_output);
        Ok(user)
    }

    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String
    ) -> Result<String, Error> {
        let table_name = "Users";

        info!("Removing user: {}", email);
        let db_client = ctx.data::<Client>().map_err(|e| {
            warn!("Failed to get db_client from context: {:?}", e);
            AppError::InternalServerError(
                "Failed to access application db_client".to_string()
            ).to_graphql_error()
        })?;

        info!("successfully created db_client: {:?}", &db_client);

        

        let remove_item_output = db_client
            .delete_item()
            .table_name(table_name)
            .key("email", AttributeValue::S(email.clone().into()))
            .send().await
            .map_err(|e| {
                warn!("Failed to delete user: {:?}", e);
                AppError::DatabaseError(
                    "Failed to delete user by email from db".to_string()
                ).to_graphql_error()
            })?;
        info!("removed item successfully, output: {:?}", &remove_item_output);
        Ok(email)
    }
}
