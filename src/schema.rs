use async_graphql::{ Context, Error, Object, SimpleObject, Result as GraphQLResult };
use aws_sdk_dynamodb::{ types::{ error::InternalServerError, AttributeValue }, Client };
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
            .table_name("Users")
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

        let user = match User::new(id, email, &password, first_name, last_name, pantry_name) {
            Ok(u) => u,
            Err(e) => {
                return Err(AppError::InternalServerError(e).to_graphql_error());
            }
        };
        let item = user.to_item();
        // Transform DynamoDB error into our AppError, then into GraphQL error
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
}
