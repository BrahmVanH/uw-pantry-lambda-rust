use std::sync::Arc;

use async_graphql::{ Context, Error, Object, SimpleObject };
use aws_sdk_dynamodb::{ types::AttributeValue, Client };
use serde::Serialize;

use crate::{ error::AppError, AppState, FailureResponse };

#[derive(SimpleObject, Serialize)]
struct User {
    username: String,
}

// GraphQL Schema
//  Query root
#[derive(Debug)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn sup(&self) -> String {
        "sup, crabs?".to_string()
    }
}

// Mutation root
#[derive(Debug)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_user(&self, ctx: &Context<'_>, username: String) -> Result<User, Error> {
        let state = match ctx.data::<AppState>() {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::InternalServerError(e.message).to_graphql_error());
            }
        };

        let username_av = AttributeValue::S(username.clone());

        let _resp = state.db_client
            .put_item()
            .table_name("test")
            .item("username", username_av)
            .send().await
            .map_err(|err| {
                FailureResponse {
                    body: "Error in putting new user in db".to_owned(),
                }
            });

        Ok(User {
            username,
        })
    }
}
