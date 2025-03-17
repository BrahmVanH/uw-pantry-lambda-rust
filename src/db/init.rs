use aws_sdk_dynamodb::operation::list_tables::ListTablesOutput;
use aws_sdk_dynamodb::types::TableClass;
use aws_sdk_dynamodb::{ Client, Error };
use aws_sdk_dynamodb::types::{
    AttributeDefinition,
    BillingMode,
    KeySchemaElement,
    KeyType,
    GlobalSecondaryIndex,
    Projection,
    ProjectionType,
    ScalarAttributeType,
};

pub async fn ensure_tables_exist(client: &Client) -> Result<(), Error> {
    let tables = client.list_tables().send().await?;
    ensure_users_table_exists(tables, client).await?;

    Ok(())
}

async fn ensure_users_table_exists(tables: ListTablesOutput, client: &Client) -> Result<(), Error> {
    let table_name = "Users";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    println!("Creating '{}' table...", table_name);

    // Create the table
    let _response = client
        .create_table()
        .table_name(table_name)
        .billing_mode(BillingMode::PayPerRequest) // On-demand capacity

        // Primary key
        .key_schema(
            KeySchemaElement::builder().attribute_name("id").key_type(KeyType::Hash).build()?
        )

        // Attribute definitions
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(ScalarAttributeType::S)
                .build()?
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("email")
                .attribute_type(ScalarAttributeType::S)
                .build()?
        )
        // Create a GSI for email lookups
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("EmailIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("email")
                        .key_type(KeyType::Hash)
                        .build()?
                )
                .projection(Projection::builder().projection_type(ProjectionType::All).build())
                .build()?
        )

        .send().await?;

    println!("Table '{}' created successfully", table_name);

    Ok(())
}
