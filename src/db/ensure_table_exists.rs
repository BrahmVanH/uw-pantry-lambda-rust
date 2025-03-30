//! DynamoDB table definitions and creation module.
//!
//! This module contains detailed definitions for all DynamoDB tables used in the application.
//! It provides functions to create tables with appropriate keys, indexes, and configurations
//! to support the data access patterns required by the application.

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

/// Helper function to simplify error handling during DynamoDB resource creation.
///
/// This function wraps the builder pattern results with proper error context.
///
/// # Type Parameters
///
/// * `T` - The success result type from a builder operation
/// * `E` - The error type from a builder operation that implements Display
///
/// # Arguments
///
/// * `builder_result` - Result from a DynamoDB builder operation
/// * `context` - Error context to include in case of failure
///
/// # Returns
///
/// * `Result<T, AppError>` - The original success value or a DatabaseError with context
fn build<T, E>(builder_result: Result<T, E>, context: &str) -> Result<T, AppError>
    where E: fmt::Display
{
    builder_result.map_err(|e| AppError::DatabaseError(format!("{}: {:?}", context, e.to_string())))
}

/// Creates the PantrySystem table using a single-table design pattern.
///
/// This table uses composite primary keys (PK, SK) and multiple GSIs to support
/// various access patterns efficiently. The design follows DynamoDB best practices
/// for flexible, efficient querying with minimal table scans.
///
/// # Primary Key Structure
/// * Partition Key (PK): Entity type prefix + ID (e.g., "PANTRY#123", "USER#456")
/// * Sort Key (SK): Entity metadata or relationship (e.g., "PROFILE", "PANTRY#123")
///
/// # Global Secondary Indexes
/// * UserAccessIndex: Find pantries a user can access
/// * PantryManagementIndex: Find users with specific access levels for a pantry
/// * SelfManagedPantryIndex: Find all self-managed pantries
/// * EmailLookupIndex: Look up users by email address
///
/// # Arguments
///
/// * `tables` - List of existing tables to check if this one already exists
/// * `client` - DynamoDB client for AWS API operations
///
/// # Returns
///
/// * `Result<(), AppError>` - Success or a database error with context
pub async fn pantry_system(tables: &ListTablesOutput, client: &Client) -> Result<(), AppError> {
    let table_name = "PantrySystem";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    // Define attribute definitions
    let ad_pk = build(
        AttributeDefinition::builder()
            .attribute_name("PK")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build PK attribute definition"
    )?;

    let ad_sk = build(
        AttributeDefinition::builder()
            .attribute_name("SK")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build SK attribute definition"
    )?;

    let ad_user_id = build(
        AttributeDefinition::builder()
            .attribute_name("USER_ID")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build USER_ID attribute definition"
    )?;

    let ad_access_level = build(
        AttributeDefinition::builder()
            .attribute_name("access_level")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build access_level attribute definition"
    )?;

    let ad_is_self_managed = build(
        AttributeDefinition::builder()
            .attribute_name("is_self_managed")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build is_self_managed attribute definition"
    )?;

    let ad_email = build(
        AttributeDefinition::builder()
            .attribute_name("email")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build email attribute definition"
    )?;

    // Define key schema for table
    let ks_pk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Hash).build(),
        "Failed to build PK key schema"
    )?;

    let ks_sk = build(
        KeySchemaElement::builder().attribute_name("SK").key_type(KeyType::Range).build(),
        "Failed to build SK key schema"
    )?;

    // Define GSI 1: User Access Index - For finding pantries a user can access
    let gsi1_pk = build(
        KeySchemaElement::builder().attribute_name("USER_ID").key_type(KeyType::Hash).build(),
        "Failed to build GSI1 PK"
    )?;

    let gsi1_sk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Range).build(),
        "Failed to build GSI1 SK"
    )?;

    let gsi1 = build(
        GlobalSecondaryIndex::builder()
            .index_name("UserAccessIndex")
            .key_schema(gsi1_pk)
            .key_schema(gsi1_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build GSI1"
    )?;

    // Define GSI 2: Pantry Management Index - For finding users with specific access levels
    let gsi2_pk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Hash).build(),
        "Failed to build GSI2 PK"
    )?;

    let gsi2_sk = build(
        KeySchemaElement::builder().attribute_name("access_level").key_type(KeyType::Range).build(),
        "Failed to build GSI2 SK"
    )?;

    let gsi2 = build(
        GlobalSecondaryIndex::builder()
            .index_name("PantryManagementIndex")
            .key_schema(gsi2_pk)
            .key_schema(gsi2_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build GSI2"
    )?;

    // Define GSI 3: Self-Managed Pantry Index - For finding all self-managed pantries
    let gsi3_pk = build(
        KeySchemaElement::builder()
            .attribute_name("is_self_managed")
            .key_type(KeyType::Hash)
            .build(),
        "Failed to build GSI3 PK"
    )?;

    let gsi3_sk = build(
        KeySchemaElement::builder().attribute_name("PK").key_type(KeyType::Range).build(),
        "Failed to build GSI3 SK"
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

    // Create the table with proper error handling
    let response = client
        .create_table()
        .table_name("PantrySystem")
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(ad_pk)
        .attribute_definitions(ad_sk)
        .attribute_definitions(ad_user_id)
        .attribute_definitions(ad_access_level)
        .attribute_definitions(ad_is_self_managed)
        .attribute_definitions(ad_email)
        .key_schema(ks_pk)
        .key_schema(ks_sk)
        .global_secondary_indexes(gsi1)
        .global_secondary_indexes(gsi2)
        .global_secondary_indexes(gsi3)
        .global_secondary_indexes(gsi4)
        .send().await
        .map_err(|e|
            AppError::DatabaseError(
                format!("Failed to create {} table: {:?}", table_name, e.to_string())
            )
        )?;

    println!("PantrySystem table created: {:?}", response);
    Ok(())
}

/// Creates a dedicated Users table for a multi-table design approach.
///
/// This table stores user information with a primary key on user_id and
/// includes global secondary indexes for email lookups and role-based queries.
///
/// # Primary Key Structure
/// * Partition Key: user_id (UUID)
///
/// # Global Secondary Indexes
/// * EmailIndex: Find users by email address (for authentication)
/// * RoleIndex: Find users by role (for administrative functions)
///
/// # Arguments
///
/// * `tables` - List of existing tables to check if this one already exists
/// * `client` - DynamoDB client for AWS API operations
///
/// # Returns
///
/// * `Result<(), AppError>` - Success or a database error with context
pub async fn users(tables: &ListTablesOutput, client: &Client) -> Result<(), AppError> {
    let table_name = "Users";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    // Define attribute definitions
    let ad_user_id = build(
        AttributeDefinition::builder()
            .attribute_name("user_id")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build user_id attribute definition"
    )?;

    let ad_email = build(
        AttributeDefinition::builder()
            .attribute_name("email")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build email attribute definition"
    )?;

    let ad_role = build(
        AttributeDefinition::builder()
            .attribute_name("role")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build role attribute definition"
    )?;

    // Define key schema for table
    let ks_user_id = build(
        KeySchemaElement::builder().attribute_name("user_id").key_type(KeyType::Hash).build(),
        "Failed to build user_id key schema"
    )?;

    // Define GSI 1: Email Lookup Index
    let gsi1_pk = build(
        KeySchemaElement::builder().attribute_name("email").key_type(KeyType::Hash).build(),
        "Failed to build Email GSI PK"
    )?;

    let gsi1 = build(
        GlobalSecondaryIndex::builder()
            .index_name("EmailIndex")
            .key_schema(gsi1_pk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build EmailIndex GSI"
    )?;

    // Define GSI 2: Role Index
    let gsi2_pk = build(
        KeySchemaElement::builder().attribute_name("role").key_type(KeyType::Hash).build(),
        "Failed to build Role GSI PK"
    )?;

    let gsi2 = build(
        GlobalSecondaryIndex::builder()
            .index_name("RoleIndex")
            .key_schema(gsi2_pk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build RoleIndex GSI"
    )?;

    // Create the table with proper error handling
    let response = client
        .create_table()
        .table_name("Users")
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(ad_user_id)
        .attribute_definitions(ad_email)
        .attribute_definitions(ad_role)
        .key_schema(ks_user_id)
        .global_secondary_indexes(gsi1)
        .global_secondary_indexes(gsi2)
        .send().await
        .map_err(|e|
            AppError::DatabaseError(
                format!("Failed to create {} table: {:?}", table_name, e.to_string())
            )
        )?;

    println!("Users table created: {:?}", response);
    Ok(())
}

/// Creates a dedicated Pantries table for a multi-table design approach.
///
/// This table stores information about food pantries, including
/// location, contact details, and operational settings.
///
/// # Primary Key Structure
/// * Partition Key: pantry_id (UUID)
///
/// # Global Secondary Indexes
/// * SelfManagedIndex: Identifies self-managed vs. centrally managed pantries
///
/// # Arguments
///
/// * `tables` - List of existing tables to check if this one already exists
/// * `client` - DynamoDB client for AWS API operations
///
/// # Returns
///
/// * `Result<(), AppError>` - Success or a database error with context
pub async fn pantries(tables: &ListTablesOutput, client: &Client) -> Result<(), AppError> {
    let table_name = "Pantries";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    // Define attribute definitions
    let ad_pantry_id = build(
        AttributeDefinition::builder()
            .attribute_name("pantry_id")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build pantry_id attribute definition"
    )?;

    let ad_is_self_managed = build(
        AttributeDefinition::builder()
            .attribute_name("is_self_managed")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build is_self_managed attribute definition"
    )?;

    // Define key schema for table
    let ks_pantry_id = build(
        KeySchemaElement::builder().attribute_name("pantry_id").key_type(KeyType::Hash).build(),
        "Failed to build pantry_id key schema"
    )?;

    // Define GSI 1: Self-Managed Index
    let gsi1_pk = build(
        KeySchemaElement::builder()
            .attribute_name("is_self_managed")
            .key_type(KeyType::Hash)
            .build(),
        "Failed to build Self-Managed GSI PK"
    )?;

    let gsi1 = build(
        GlobalSecondaryIndex::builder()
            .index_name("SelfManagedIndex")
            .key_schema(gsi1_pk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build SelfManagedIndex GSI"
    )?;

    // Create the table with proper error handling
    let response = client
        .create_table()
        .table_name("Pantries")
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(ad_pantry_id)
        .attribute_definitions(ad_is_self_managed)
        .key_schema(ks_pantry_id)
        .global_secondary_indexes(gsi1)
        .send().await
        .map_err(|e|
            AppError::DatabaseError(
                format!("Failed to create {} table: {:?}", table_name, e.to_string())
            )
        )?;

    println!("Pantries table created: {:?}", response);
    Ok(())
}

/// Creates a PantryAccess table for managing user-pantry access relationships.
///
/// This table implements an access control system where users can have
/// different levels of access to different pantries. It supports
/// queries for pantry administrators, contact agents, and viewing which
/// pantries a user can access.
///
/// # Primary Key Structure
/// * Partition Key: pantry_id (UUID)
/// * Sort Key: user_id (UUID)
///
/// # Global Secondary Indexes
/// * UserAccessIndex: Find all pantries a specific user can access
/// * AccessLevelIndex: Find users with specific access levels for a pantry
/// * ContactAgentIndex: Find contact agents for a pantry
///
/// # Arguments
///
/// * `tables` - List of existing tables to check if this one already exists
/// * `client` - DynamoDB client for AWS API operations
///
/// # Returns
///
/// * `Result<(), AppError>` - Success or a database error with context
pub async fn pantry_access(tables: &ListTablesOutput, client: &Client) -> Result<(), AppError> {
    let table_name = "PantryAccess";

    // Check if table already exists
    if tables.table_names().contains(&table_name.to_string()) {
        println!("Table '{}' already exists", table_name);
        return Ok(());
    }

    // Define attribute definitions
    let ad_pantry_id = build(
        AttributeDefinition::builder()
            .attribute_name("pantry_id")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build pantry_id attribute definition"
    )?;

    let ad_user_id = build(
        AttributeDefinition::builder()
            .attribute_name("user_id")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build user_id attribute definition"
    )?;

    let ad_access_level = build(
        AttributeDefinition::builder()
            .attribute_name("access_level")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build access_level attribute definition"
    )?;

    let ad_is_contact_agent = build(
        AttributeDefinition::builder()
            .attribute_name("is_contact_agent")
            .attribute_type(ScalarAttributeType::S)
            .build(),
        "Failed to build is_contact_agent attribute definition"
    )?;

    // Define key schema for table - composite key of pantry_id and user_id
    let ks_pantry_id = build(
        KeySchemaElement::builder().attribute_name("pantry_id").key_type(KeyType::Hash).build(),
        "Failed to build pantry_id key schema"
    )?;

    let ks_user_id = build(
        KeySchemaElement::builder().attribute_name("user_id").key_type(KeyType::Range).build(),
        "Failed to build user_id key schema"
    )?;

    // Define GSI 1: User Access Index
    let gsi1_pk = build(
        KeySchemaElement::builder().attribute_name("user_id").key_type(KeyType::Hash).build(),
        "Failed to build User Access GSI PK"
    )?;

    let gsi1_sk = build(
        KeySchemaElement::builder().attribute_name("pantry_id").key_type(KeyType::Range).build(),
        "Failed to build User Access GSI SK"
    )?;

    let gsi1 = build(
        GlobalSecondaryIndex::builder()
            .index_name("UserAccessIndex")
            .key_schema(gsi1_pk)
            .key_schema(gsi1_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build UserAccessIndex GSI"
    )?;

    // Define GSI 2: Access Level Index
    let gsi2_pk = build(
        KeySchemaElement::builder().attribute_name("pantry_id").key_type(KeyType::Hash).build(),
        "Failed to build Access Level GSI PK"
    )?;

    let gsi2_sk = build(
        KeySchemaElement::builder().attribute_name("access_level").key_type(KeyType::Range).build(),
        "Failed to build Access Level GSI SK"
    )?;

    let gsi2 = build(
        GlobalSecondaryIndex::builder()
            .index_name("AccessLevelIndex")
            .key_schema(gsi2_pk)
            .key_schema(gsi2_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build AccessLevelIndex GSI"
    )?;

    // Define GSI 3: Contact Agent Index
    let gsi3_pk = build(
        KeySchemaElement::builder().attribute_name("pantry_id").key_type(KeyType::Hash).build(),
        "Failed to build Contact Agent GSI PK"
    )?;

    let gsi3_sk = build(
        KeySchemaElement::builder()
            .attribute_name("is_contact_agent")
            .key_type(KeyType::Range)
            .build(),
        "Failed to build Contact Agent GSI SK"
    )?;

    let gsi3 = build(
        GlobalSecondaryIndex::builder()
            .index_name("ContactAgentIndex")
            .key_schema(gsi3_pk)
            .key_schema(gsi3_sk)
            .projection(Projection::builder().projection_type(ProjectionType::All).build())
            .build(),
        "Failed to build ContactAgentIndex GSI"
    )?;

    // Create the table with proper error handling
    let response = client
        .create_table()
        .table_name("PantryAccess")
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(ad_pantry_id)
        .attribute_definitions(ad_user_id)
        .attribute_definitions(ad_access_level)
        .attribute_definitions(ad_is_contact_agent)
        .key_schema(ks_pantry_id)
        .key_schema(ks_user_id)
        .global_secondary_indexes(gsi1)
        .global_secondary_indexes(gsi2)
        .global_secondary_indexes(gsi3)
        .send().await
        .map_err(|e|
            AppError::DatabaseError(
                format!("Failed to create {} table: {:?}", table_name, e.to_string())
            )
        )?;

    println!("PantryAccess table created: {:?}", response);
    Ok(())
}
