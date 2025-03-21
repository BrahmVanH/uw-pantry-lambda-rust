use async_graphql::{ Context, Object, Error };
use aws_sdk_dynamodb::{ types::AttributeValue, Client };
use tracing::{ info, warn };
use crate::models::user::User;

use uuid::Uuid;

use crate::error::AppError;

// Mutation root
#[derive(Debug)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // Creates new user in database
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
        let user = User::new(id, email, &password, first_name, last_name, pantry_name).map_err(|e|
            AppError::DatabaseError(e)
        )?;

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

    // login user using email and password
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String
    ) -> Result<String, Error> {
        let user = self.user_by_email(ctx, email);
        map_err(|e| {
            return e;
        })?;

        
    }

    // Remove user from database by email
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
