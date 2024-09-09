

use serde::Deserialize;
use serde_json::Value;

use crate::{create::create_record, find_matching::find_matching, get::get_record, update::update_record, ApiClient, IntegrationRecord};
use core::fmt::Debug;

pub struct SyncRecordData<T, C: ApiClient> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub get: GetData,
    pub create: CreateData<T, C>,
    pub update: UpdateData<T, C>,
    pub find_matching: FindMatchingData,
    pub index_matching_id: fn(json: &Value) -> Result<String, String>,
    pub deserialize: Option<fn(&Value) -> T>,
    pub to_api_client: C,
    pub to_type: String,
    pub from_token: String
}

pub struct GetData {
    pub url: String,
}

pub struct CreateData<T, C: ApiClient> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: ConstructUrl<C>,
    pub payload: ConstructPayload<T>
}

pub type ConstructPayload<T> = fn(&T, Option<String>) -> Result<Value, String>;

pub type ConstructUrl<C> = fn(client: &C, _type: &str, existing_id: &Option<String>) -> String;

pub struct UpdateData<T, C: ApiClient> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: ConstructUrl<C>,
    pub payload: ConstructPayload<T>
}

pub struct FindMatchingData {
    pub properties: Vec<String>,
    pub construct_search_url: fn(property: &str, value: &str) -> Result<String, String>,
    pub payload: Option<fn(property: &str) -> Value>,
    pub index_array: fn(json: Value) -> Value,
}


/// To sync a record from one application to another
/// Both records should implement IntegrationRecord
/// Intended use is right after receiving a webhook of a change, pass the ID and the relevant functions here to sync
pub async fn sync_record<T, C: ApiClient>(
    parameters: SyncRecordData<T, C>,
    // get_record: impl Fn(&str) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (id: &str) -> Result<Option<T>, String>
    meets_conditions: fn(record: &T) -> bool,
    // find matching should return the matching record from the other system
    // find_matching: impl Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (record: T) -> Result<Option<T>, String>
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let record: T = get_record(&parameters.get.url, parameters.deserialize, &parameters.from_token).await?;
    println!("got record: {:#?}", record);

    match meets_conditions(&record) {
        true => match find_matching::<T, C>(
            &record,
            &parameters.to_api_client,
            parameters.find_matching.properties,
            parameters.find_matching.construct_search_url,
            parameters.find_matching.payload,
            parameters.find_matching.index_array,
        ).await? {
            Some(matching_record) => update_record::<T, C>(
                parameters.update.url,
                &parameters.to_api_client,
                &parameters.to_type,
                Some((parameters.index_matching_id)(&matching_record)?),
                &(parameters.update.payload)(&record)?
            ).await?,
            None => create_record::<T, C>(
                parameters.create.url,
                &parameters.to_api_client,
                &parameters.to_type,
                None,
                &(parameters.create.payload)(&record)?
            ).await?
        },
        false => println!("Record did not meet conditions to sync")
    };

    return Ok(())
}

// Outdated:
// create_record: impl Fn(T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>
// update record takes in the record received, and the matching record in the other application
// update_record: impl Fn(T, T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>