

use serde::Deserialize;
use serde_json::Value;

use crate::{create::create_record, find_matching::find_matching, get::get_record, update::update_record, IntegrationRecord};
use core::fmt::Debug;

pub struct SyncRecordData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    // record_id: String,
    pub get: GetData,
    pub create: CreateData<T>,
    pub update: UpdateData<T>,
    pub find_matching: FindMatchingData,
    pub deserialize: Option<fn(&Value) -> T>,
    pub token: String
}

pub struct GetData {
    pub url: String,
}

pub struct CreateData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: String,
    pub payload: fn(&T) -> Value,
}

pub struct UpdateData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: String,
    payload: fn(&T) -> Value,
}

pub struct FindMatchingData {
    properties: Vec<String>,
    construct_search_url: fn(property: &str) -> String,
    payload: fn(property: &str) -> Value,
    index_array: fn(json: Value) -> Value,
}


/// To sync a record from one application to another
/// Both records should implement IntegrationRecord
/// Intended use is right after receiving a webhook of a change, pass the ID and the relevant functions here to sync
pub async fn sync_record<T>(
    parameters: SyncRecordData<T>,
    // get_record: impl Fn(&str) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (id: &str) -> Result<Option<T>, String>
    meets_conditions: fn(record: &T) -> bool,
    // find matching should return the matching record from the other system
    // find_matching: impl Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (record: T) -> Result<Option<T>, String>
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let record: T = get_record(&parameters.get.url, parameters.deserialize).await?;
    println!("got record: {:#?}", record);

    match meets_conditions(&record) {
        true => match find_matching::<T>(
            parameters.find_matching.properties,
            parameters.find_matching.construct_search_url,
            parameters.find_matching.payload,
            parameters.find_matching.index_array,
            &parameters.token
        ).await? {
            Some(_matching_record) => update_record::<T>(&parameters.update.url, &(parameters.update.payload)(&record)).await?,
            None => create_record::<T>(&parameters.create.url, &(parameters.create.payload)(&record)).await?
        },
        false => println!("Record did not meet conditions to sync")
    };

    return Ok(())
}

// Outdated:
// create_record: impl Fn(T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>
// update record takes in the record received, and the matching record in the other application
// update_record: impl Fn(T, T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>