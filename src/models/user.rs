use async_graphql::{ Context, Object, ID, Result as GraphQLResult };
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
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

// Each user is the agent of a food pantry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub pantry_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        id: String,
        email: String,
        password: &str,
        first_name: String,
        last_name: String,
        pantry_name: String
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
            pantry_name,
            created_at: now,
            updated_at: now,
        })
    }
    // Turn into and from DynamoDB User Item
    pub fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        Some(Self {
            id: item.get("id")?.as_s().ok()?.clone(),
            email: item.get("email")?.as_s().ok()?.clone(),
            password_hash: item.get("password")?.as_s().ok()?.clone(),
            first_name: item.get("first_name")?.as_s().ok()?.clone(),
            last_name: item.get("last_name")?.as_s().ok()?.clone(),
            pantry_name: item.get("email")?.as_s().ok()?.clone(),
            created_at: item
                .get("created_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or_else(|| Utc::now()),
            updated_at: item
                .get("updated_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or_else(|| Utc::now()),
        })
    }

    pub fn to_item(&self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert("id".to_string(), AttributeValue::S(self.id.clone()));
        item.insert("email".to_string(), AttributeValue::S(self.email.clone()));
        item.insert("password_hash".to_string(), AttributeValue::S(self.password_hash.clone()));
        item.insert("first_name".to_string(), AttributeValue::S(self.first_name.clone()));
        item.insert("last_name".to_string(), AttributeValue::S(self.last_name.clone()));
        item.insert("pantry_name".to_string(), AttributeValue::S(self.pantry_name.clone()));
        item.insert("created_at".to_string(), AttributeValue::S(self.created_at.to_string()));
        item.insert("updated_at".to_string(), AttributeValue::S(self.updated_at.to_string()));

        item
    }

    //  Validate password for login
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

    async fn pantry_name(&self) -> &str {
        &self.pantry_name
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
