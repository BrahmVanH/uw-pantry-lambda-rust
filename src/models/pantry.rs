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

use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
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
    pub agent_id: String,
    pub opt_status: OptStatus,
    // pub flags:
    // pub address: Address
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    ///
    ///
    /// # Returns
    ///
    /// New Pantry instance
    ///
    ///
    pub fn new(
        id: String,
        name: String,
        agent_id: String,
        opt_status: OptStatus
        // flags: ,
    ) -> Result<Self, String> {
        let now = Utc::now();

        Ok(Self {
            id,
            name,
            agent_id,
            opt_status,
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

        let agent_id = item.get("agent_id")?.as_s().ok()?.to_string();

        let opt_status_str = item.get("opt_status")?.as_s().ok()?;

        // Turns opt_status_str received on pantry from db into OptStatus enum value
        let opt_status: OptStatus = serde_json
            ::from_str::<OptStatus>(opt_status_str)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
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
            agent_id,
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

    pub fn to_item(&self) -> HashMap<String, AttributeValue> {
      let mut item = HashMap::new();

      item.insert("id".to_string(), v)
    }  
    
  }