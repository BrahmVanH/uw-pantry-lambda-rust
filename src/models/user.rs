use async_graphql::{ Context, Object, ID, Result as GraphQLResult };
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use tracing::info;
use std::collections::HashMap;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        Salt,
        SaltString,
    },
    Argon2,
};

/// Represents user in system
///
/// # Fields
///
/// * `id` - Unique identifier for user
/// * `email` - email address of user
/// * `password_hash` - hashed user password
/// * `first_name` - users first name
/// * `last_name` - users last name
/// * `pantry_id` - ID of food pantry table row where user is agent
/// * `created_at` - Date and time of creation
/// * `updated_at` - Date and Time of creation

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pantry_id: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Defines methods for User
impl User {
    /// Creates new User instance
    ///
    /// # Arguments
    ///
    /// * `id` - new user ID
    /// * `email` - user email address
    /// * `password` - user password
    /// * `first_name` - user's first name
    /// * `last_name` - user's last name
    ///
    /// # Returns
    ///
    /// New user instance

    pub fn new(
        id: String,
        email: String,
        password: &str,
        first_name: String,
        last_name: String
    ) -> Result<Self, String> {
        let now = Utc::now();

        // Generate a salt for password
        let salt = SaltString::generate(&mut OsRng);

        // Configure Argon2 with default parameters
        let argon2 = Argon2::default();

        // hash password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?
            .to_string();

        Ok(Self {
            id,
            email,
            password_hash,
            first_name,
            last_name,
            pantry_id: None,
            created_at: now,
            updated_at: now,
        })
    }
    /// Creates User instance from DynamoDB item
    ///
    /// # Arguments
    ///
    /// * `item` - The dynamo db item
    ///
    /// # Returns
    ///
    /// 'some' User if item fields match, 'none' otherwise

    pub fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        info!("calling from_item with: {:?}", &item);

        let id = item.get("id")?.as_s().ok()?.to_string();
        info!("got id: {}", id);

        let email = item.get("email")?.as_s().ok()?.to_string();
        info!("got email: {}", email);

        let password_hash = item.get("password_hash")?.as_s().ok()?.to_string();
        info!("got password hash");

        let first_name = item.get("first_name")?.as_s().ok()?.to_string();
        info!("got first_name: {}", first_name);

        let last_name = item.get("last_name")?.as_s().ok()?.to_string();
        info!("got last_name: {}", last_name);

        // Handle pantry item as optional field
        let pantry_id = item
            .get("pantry")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.to_string());

        let created_at = item
            .get("created_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or_else(|| Utc::now());

        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or_else(|| Utc::now());

        let res = Some(Self {
            id,
            email,
            password_hash,
            first_name,
            last_name,
            pantry_id,
            created_at,
            updated_at,
        });

        info!("result of from_item: {:?}", &res);
        res
    }

    /// Creates DynamoDB item from User instance
    ///
    /// # Arguments
    ///
    /// * `self` - borrowed instance of self
    ///
    /// # Returns
    ///
    ///   HashMap representing DB item for User instance

    pub fn to_item(&self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(self.id.clone()));
        item.insert("email".to_string(), AttributeValue::S(self.email.clone()));
        item.insert("password_hash".to_string(), AttributeValue::S(self.password_hash.clone()));
        item.insert("first_name".to_string(), AttributeValue::S(self.first_name.clone()));
        item.insert("last_name".to_string(), AttributeValue::S(self.last_name.clone()));
        match &self.pantry_id {
            Some(id) => {
                item.insert("pantry_id".to_string(), AttributeValue::S(id.clone()));
            }
            None => (),
        }
        item.insert("created_at".to_string(), AttributeValue::S(self.created_at.to_string()));
        item.insert("updated_at".to_string(), AttributeValue::S(self.updated_at.to_string()));

        item
    }

    /// Verifies that given password matches the parsed password hash on given user
    ///
    /// # Arguments
    ///
    /// * `self` - borrowed instance of self
    ///
    /// # Returns
    ///
    ///   HashMap representing DB item for Pantry instance

    pub fn verify_password(&self, password: &str) -> bool {
        // parse password hash
        let parsed_hash = match PasswordHash::new(&self.password_hash) {
            Ok(hash) => hash,
            Err(e) => {
                return false;
            }
        };

        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }

    pub fn update_password(&mut self, password: &str) -> Result<(), String> {
        // generate salt
        let salt = SaltString::generate(OsRng);

        let argon2 = Argon2::default();

        self.password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?
            .to_string();

        self.updated_at = Utc::now();

        Ok(())
    }
}

// GraphQL Implementation
#[Object]
impl User {
    async fn id(&self) -> ID {
        ID(self.id.clone())
    }

    async fn email(&self) -> &str {
        &self.email
    }

    /// User may not always have a pantry value or pantry field at all
    async fn pantry_name(&self) -> &str {
        match &self.pantry_id {
            Some(id) => id,
            None => "",
        }
    }
    async fn first_name(&self) -> &str {
        &self.first_name
    }
    async fn last_name(&self) -> &str {
        &self.last_name
    }
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    async fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
