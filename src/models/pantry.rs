//! ID
//! name
//! address: Address
//! agent: ID of user
//! opt-in-status: enum t1, t2, t3
//! flags: Vac<PantryFeatureFlag>
//! created at
//! updated at
//!
//!
//!
//!
//! Address: { geo: x, y}
//!
//! Notes:
//!   Going to need a function that turns vec of pantry db items to vec of geojson features
//!
//!

use std::{ collections::HashMap };

use async_graphql::{ Object, SimpleObject };
use aws_sdk_dynamodb::{ types::AttributeValue };
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use tracing::info;

use crate::error::AppError;

/// Represent variant of Opt-Status for pantry
///
/// # Variants
///
/// * `T1` - opted-out; Pantry does not have feature flags or inventory
/// * `T2` - opted-in w/ flags; Pantry will have feature flags and will appear
///           in Pantry Hub in UI; Pantry does not have inventory
/// * `T3` - opted-in fully; Pantry will have feature flags and inventory
///

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum OptStatus {
    T1,
    T2,
    T3,
}

impl OptStatus {
    fn to_string(&self) -> String {
        match self {
            OptStatus::T1 => "T1".to_string(),
            OptStatus::T2 => "T2".to_string(),
            OptStatus::T3 => "T3".to_string(),
        }
    }
    fn to_str(&self) -> &str {
        match self {
            OptStatus::T1 => "T1",
            OptStatus::T2 => "T2",
            OptStatus::T3 => "T3",
        }
    }
    fn from_string(s: &str) -> Result<OptStatus, AppError> {
        match s {
            "T1" => Ok(Self::T1),
            "T2" => Ok(Self::T2),
            "T3" => Ok(Self::T3),
            _ => {
                return Err(
                    AppError::DatabaseError("Invalid opt status from pantry item".to_string())
                );
            }
        }
    }
}

/// Represents a Food Pantry involved in program
///
/// # Fields
///
/// * `id` - Unique identifier for the pantry
/// * `name` - Name of food pantry
/// * `agent` - ID of user designated as agent for pantry
/// * `opt_status` - Value from OptStatus enum representing involvement level in program
/// * `flags` - Flags denoting particulars about food pantry and requirements to receive services
/// * `address` - Address of Pantry
/// * `created_at` - Date and time of creation
/// * `updated_at` - Date and time of last update

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pantry {
    pub id: String,
    pub name: String,
    pub is_self_managed: String,
    pub opt_status: OptStatus,
    pub phone: String,
    pub email: String,
    // pub flags:
    pub address: Address,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a physical street address using format for united states
///
/// # Fields
///
/// * `street` - street address with number and street name
/// * `unit` - optional unit specification for address
/// * `city` - the city
/// * `state` - the state
/// * `zipcode` - zipcode of address
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub unit: Option<String>,
    pub city: String,
    pub state: String,
    pub zipcode: String,
}

/// Defines methods for Pantry
impl Pantry {
    /// Creates new Pantry instance
    ///
    /// # Arguments
    ///
    /// * `id` - Unique Uuid
    /// * `name` - Name of Pantry
    /// * `agent_id` - ID string of User in DB assigned as agent
    /// * `opt_status` - enum OptStatus
    /// * `flags` -
    /// * `address` - pantry's physical address
    /// * `is_self_managed` - bool representing whether or not user associated with pantry
    ///                         will be managing the pantry on this platform
    /// * `phone` - phone number of pantry
    /// * `email` - email address of pantry
    ///
    /// # Returns
    ///
    /// New Pantry instance
    ///
    ///
    pub fn new(
        id: String,
        name: String,
        opt_status: OptStatus,
        address: Address,
        is_self_managed: bool,
        phone: String,
        email: String
        // flags: ,
    ) -> Result<Self, String> {
        let now = Utc::now();

        let is_self_managed_str = match is_self_managed {
            true => "true",
            false => "false",
        };

        Ok(Self {
            id,
            name,
            opt_status,
            address,
            is_self_managed: is_self_managed_str.to_string(),
            phone,
            email,
            created_at: now,
            updated_at: now,
        })
    }
    /// Creates Pantry instance from DynamoDB item
    ///
    /// # Arguments
    ///
    /// * `item` - The dynamo db item
    ///
    /// # Returns
    ///
    /// 'some' Pantry if item fields match, 'none' otherwise

    pub fn from_item(item: &HashMap<String, AttributeValue>) -> Option<Self> {
        info!("calling from_item with: {:?}", &item);

        let id = item.get("id")?.as_s().ok()?.to_string();

        let name = item.get("name")?.as_s().ok()?.to_string();

        // let agent_id = item.get("agent_id")?.as_s().ok()?.to_string();
        let item_address = item.get("address")?.as_m().ok()?;
        let address = Address {
            street: item_address.get("street")?.as_s().ok()?.to_string(),
            unit: item_address.get("unit")?.as_s().ok().cloned(),
            city: item_address.get("city")?.as_s().ok()?.to_string(),
            state: item_address.get("state")?.as_s().ok()?.to_string(),
            zipcode: item_address.get("zipcode")?.as_s().ok()?.to_string(),
        };

        let is_self_managed = item.get("is_self_managed")?.as_s().ok()?.to_string();

        let phone = item.get("phone")?.as_s().ok()?.to_string();

        let email = item.get("email")?.as_s().ok()?.to_string();

        let opt_status_str = item.get("opt_status")?.as_s().ok()?;

        // Turns opt_status_str received on pantry from db into OptStatus enum value
        let opt_status = OptStatus::from_string(&opt_status_str)
            .map_err(|e| e)
            .ok()?;

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
            name,
            address,
            is_self_managed,
            phone,
            email,
            opt_status,
            created_at,
            updated_at,
        });

        info!("result of from_item on pantry: {:?}", res);
        res
    }

    /// Creates DynamoDB item from Pantry instance
    ///
    /// # Arguments
    ///
    /// * `self` - borrowed instance of self
    ///
    /// # Returns
    ///
    ///   HashMap representing DB item for Pantry instance
    ///
    /// # Errors
    ///
    /// Returns an error if the serde_json::to_string() function does not complete
    /// successfully on self.opt_status

    pub fn to_item(&self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        let mut address = HashMap::new();

        let opt_status_string = serde_json
            ::to_string::<OptStatus>(&self.opt_status)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
            .ok();

        item.insert("id".to_string(), AttributeValue::S(self.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(self.name.clone()));
        item.insert("is_self_managed".to_string(), AttributeValue::S(self.is_self_managed.clone()));
        item.insert("phone".to_string(), AttributeValue::S(self.phone.clone()));
        item.insert("email".to_string(), AttributeValue::S(self.email.clone()));

        // convert nested address fields to Attribute Values and put in address map
        address.insert("street".to_string(), AttributeValue::S(self.address.street.clone()));

        // unit is optional, the field will not be created in the db item if not present on struct
        if let Some(unit) = &self.address.unit {
            address.insert("unit".to_string(), AttributeValue::S(unit.clone()));
        }

        address.insert("city".to_string(), AttributeValue::S(self.address.city.clone()));
        address.insert("state".to_string(), AttributeValue::S(self.address.state.clone()));

        address.insert("zipcode".to_string(), AttributeValue::S(self.address.zipcode.clone()));

        // insert address map into item map
        item.insert("address".to_string(), AttributeValue::M(address));

        if let Some(s) = opt_status_string {
            item.insert("opt_status".to_string(), AttributeValue::S(s));
        }

        item.insert("created_at".to_string(), AttributeValue::S(self.created_at.to_string()));
        item.insert("updated_at".to_string(), AttributeValue::S(self.updated_at.to_string()));

        item
    }
}

#[Object]
impl Pantry {
    async fn id(&self) -> &str {
        &self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn is_self_managed(&self) -> &str {
        &self.is_self_managed
    }
    async fn opt_status(&self) -> &str {
        OptStatus::to_str(&self.opt_status)
    }
    async fn phone(&self) -> &str {
        &self.phone
    }
    async fn email(&self) -> &str {
        &self.email
    }

    async fn address(&self) -> &Address {
        &self.address
    }

    async fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    async fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[Object]
impl Address {
    async fn street(&self) -> &str {
        &self.street
    }
    async fn unit(&self) -> &str {
        match &self.unit {
            Some(u) => u,
            None => "",
        }
    }
    async fn city(&self) -> &str {
        &self.city
    }
    async fn state(&self) -> &str {
        &self.state
    }
    async fn zipcode(&self) -> &str {
        &self.zipcode
    }
}
