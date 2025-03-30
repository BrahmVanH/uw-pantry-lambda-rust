use core::fmt;

use aws_sdk_dynamodb::{
    Client,
    Error,
    operation::list_tables::ListTablesOutput,
    types::{
        AttributeDefinition,
        BillingMode,
        KeySchemaElement,
        KeyType,
        GlobalSecondaryIndex,
        Projection,
        ProjectionType,
        ScalarAttributeType,
    },
};

use crate::error::AppError;

fn build<T, E>(builder_result: Result<T, E>, context: &str) -> Result<T, AppError>
    where E: fmt::Display
{
    builder_result.map_err(|e| AppError::DatabaseError(e.to_string()))
}

/// Ensures the Users table exists with the correct schema.
///
/// Creates the Users table if it doesn't exist. The table includes:
/// - Primary partition key: id (String)
/// - Global Secondary Index: EmailIndex (for lookups by email)
/// - On-demand (pay-per-request) billing model
///
/// # Arguments
///
/// * `tables` - ListTablesOutput containing existing tables
/// * `client` - A reference to the DynamoDB client
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the table exists or was created successfully,
///                         Err if an AWS error occurred
pub async fn usertt(tables: ListTablesOutput, client: &Client) -> Result<(), Error> {
    let table_name = "Users";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    println!("Creating '{}' table...", table_name);

    // Create the table with the following configuration:
    // - Primary key: id (String, Hash/Partition key)
    // - GSI: EmailIndex with email as the Hash key
    // - On-demand capacity for cost optimization
    let _response = client
        .create_table()
        .table_name(table_name)
        .billing_mode(BillingMode::PayPerRequest) // On-demand capacity for unpredictable workloads

        // Primary key definition - using id as the partition key
        .key_schema(
            KeySchemaElement::builder().attribute_name("id").key_type(KeyType::Hash).build()?
        )

        // Define attributes used in keys and indexes
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("id")
                .attribute_type(ScalarAttributeType::S) // String type
                .build()?
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("email")
                .attribute_type(ScalarAttributeType::S) // String type
                .build()?
        )

        // Create a Global Secondary Index for efficient email lookups
        // Note: This is crucial for login functionality
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("EmailIndex")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("email")
                        .key_type(KeyType::Hash)
                        .build()?
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::All) // Include all attributes in the index
                        .build()
                )
                .build()?
        )

        .send().await?;

    println!("Table '{}' created successfully", table_name);

    Ok(())
}

async fn users(tables: ListTablesOutput, client: &Client) -> Result<(), AppError> {
  let table_name = "Users";

  
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    
}

async fn pantry_system(tables: ListTablesOutput, client: &Client) -> Result<(), AppError> {
    let table_name = "Pantries";

    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    let ad_pk = build(
        AttributeDefinition::builder()
            .attribute_name("PK")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failted to build PK attributes definition"
    )?;

    let ad_sk = build(
        AttributeDefinition::builder()
            .attribute_name("SK")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build  attribute definition"
    )?;

    let ad_user_id = build(
        AttributeDefinition::builder()
            .attribute_name("USER_ID")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build  attribute definition"
    )?;

    let ad_access_level = build(
        AttributeDefinition::builder()
            .attribute_name("access_level")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build  attribute definition"
    )?;

    let ad_is_self_managed = build(
        AttributeDefinition::builder()
            .attribute_name("is_self_managed")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build  attribute definition"
    )?;

    let ks_pk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Hash).build(),
        "Failed to build  attribute definition"
    )?;

    let ks_sk = build(
        KeySchemaElement::builder().attribute_name("SK").key_type(KeyType::Hash).build(),
        "Failed to build  attribute definition"
    )?;

    let gsi1_pk = build(
        KeySchemaElement::builder().attribute_name("USER_ID").key_type(KeyType::Hash).build(),
        "Failed to build  key schema element"
    )?;

    let gsi1_sk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Hash).build(),
        "Failed to build  key schema element"
    )?;

    let gsi1 = build(
        GlobalSecondaryIndex::builder()
            .index_name("UserAccessIndex")
            .key_schema(gsi1_pk)
            .key_schema(gsi1_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build UserAccessIndex GSI1"
    )?;

    let gsi2_pk = build(
        KeySchemaElement::builder()
            .attribute_name("PK") // PANTRY#<id>
            .key_type(KeyType::Hash)
            .build(),
        "Failed to build PK GSI2"
    )?;

    let gsi2_sk = build(
        KeySchemaElement::builder().attribute_name("access_level").key_type(KeyType::Range).build(),
        "Failed to build access_level GSI2"
    )?;

    let gsi2 = build(
        GlobalSecondaryIndex::builder()
            .index_name("PantryManagementIndex")
            .key_schema(gsi2_pk)
            .key_schema(gsi2_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build PantryManagementIndex GSI2"
    )?;

    let gsi3_pk = build(
        KeySchemaElement::builder()
            .attribute_name("is_self_managed")
            .key_type(KeyType::Hash)
            .build(),
        "Failed to build is_self_managed GSI3"
    )?;

    let gsi3_sk = build(
        KeySchemaElement::builder()
            .attribute_name("PK") // PANTRY#<id>
            .key_type(KeyType::Range)
            .build(),
        "Failed to build PK GSI3"
    )?;

    let gsi3 = build(
        GlobalSecondaryIndex::builder()
            .index_name("SelfManagedPantryIndex")
            .key_schema(gsi3_pk)
            .key_schema(gsi3_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build GSI3"
    )?;
    // Define GSI 4: Email Lookup Index - For finding users by email
    let gsi4_pk = build(
        KeySchemaElement::builder().attribute_name("email").key_type(KeyType::Hash).build(),
        "Failed to build GSI4 PK"
    )?;

    let gsi4 = build(
        GlobalSecondaryIndex::builder()
            .index_name("EmailLookupIndex")
            .key_schema(gsi4_pk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build GSI4"
    )?;

    // Create the table
    let response = client
        .create_table()
        .table_name("PantrySystem")
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(ad_pk)
        .attribute_definitions(ad_sk)
        .attribute_definitions(ad_user_id)
        .attribute_definitions(ad_access_level)
        .attribute_definitions(ad_is_self_managed)
        .key_schema(ks_pk)
        .key_schema(ks_sk)
        .global_secondary_indexes(gsi1)
        .global_secondary_indexes(gsi2)
        .global_secondary_indexes(gsi3)
        .global_secondary_indexes(gsi4)
        .send().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    println!("Table created: {:?}", response);
    Ok(())
}
