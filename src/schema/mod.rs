pub mod mutation;
pub mod query;
pub mod types;

use async_graphql::{ EmptySubscription, Schema, SchemaBuilder };

use aws_sdk_dynamodb::Client;
pub use query::QueryRoot;
pub use mutation::MutationRoot;
pub use types::*;

pub type AppSchema = Schema<EmptySubscription, MutationRoot, QueryRoot>;

pub fn build_schema(db_client: &Client) -> Schema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).data(db_client.clone()).finish()
}
