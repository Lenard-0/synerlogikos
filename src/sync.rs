

use serde::Deserialize;
use serde_json::Value;

use crate::{create::create_record, find_matching::find_matching, get::get_record, update::update_record, ApiClient, IntegrationRecord};
use core::fmt::Debug;
use std::{future::Future, pin::Pin};

pub struct SyncRecordData<T, From: ApiClient, To: ApiClient> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub get: GetData,
    pub create: CreateData<T, To>,
    pub update: UpdateData<T, To>,
    pub find_matching: FindMatchingData,
    pub index_matching_id: fn(json: &Value) -> Result<String, String>,
    pub deserialize: Option<fn(&Value) -> T>,
    pub from_api_client: From,
    pub to_api_client: To,
    pub to_type: String,
    pub get_matching_record_id_for_association: Option<fn(matching_record: Value) -> Pin<Box<dyn Future<Output = Result<String, String>>>>>,
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
    pub payload: Option<fn(property: &str, value: &str) -> Value>,
    pub index_array: fn(json: Value) -> Value,
}


/// To sync a record from one application to another
/// Both records should implement IntegrationRecord
/// Intended use is right after receiving a webhook of a change, pass the ID and the relevant functions here to sync
pub async fn sync_record<T, From: ApiClient, To: ApiClient>(
    parameters: SyncRecordData<T, From, To>,
    meets_conditions: Option<impl Fn(T, Value, From) -> Pin<Box<dyn Future<Output = Result<Option<Option<Value>>, String>>>>>,
    // find matching should return the matching record from the other system
    // find_matching: impl Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (record: T) -> Result<Option<T>, String>
) -> Result<(), String> where T: IntegrationRecord + Clone + Debug + for<'de> Deserialize<'de> {
    let (record, json) = get_record(&parameters.get.url, parameters.deserialize.clone(), &parameters.from_api_client.access_token()).await?;
    println!("got record: {:#?}", record);

    return match meets_conditions {
        Some(meets_conditions) => match meets_conditions(record.clone(), json, parameters.from_api_client.clone()).await? {
            Some(opt_company) => actualise_sync(parameters, record, opt_company).await,
            None => {
                println!("Record did not meet conditions to sync");
                return Ok(())
            }
        },
        None => actualise_sync(parameters, record, None).await
    }
}

async fn actualise_sync<T, From: ApiClient, To: ApiClient>(
    parameters: SyncRecordData<T, From, To>,
    record: T,
    opt_comp: Option<Value>,
    // find matching should return the matching record from the other system
    // find_matching: impl Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (record: T) -> Result<Option<T>, String>
) -> Result<(), String> where T: IntegrationRecord + Clone + Debug + for<'de> Deserialize<'de> {
    Ok(match find_matching::<T, To>(
        &record,
        &parameters.to_api_client,
        parameters.find_matching.properties,
        parameters.find_matching.construct_search_url,
        parameters.find_matching.payload,
        parameters.find_matching.index_array,
    ).await? {
        Some(matching_record) => update_record::<T, To>(
            parameters.update.url,
            &parameters.to_api_client,
            &parameters.to_type,
            Some((parameters.index_matching_id)(&matching_record)?),
            &(parameters.create.payload)(&record, match parameters.get_matching_company_id_for_association {
                Some(get_matching_company_id_for_association) => match opt_comp {
                    Some(opt_comp) => Some(get_matching_company_id_for_association(opt_comp).await?),
                    None => None
                },
                None => None
            })?
        ).await?,
        None => create_record::<T, To>(
            parameters.create.url,
            &parameters.to_api_client,
            &parameters.to_type,
            None,
            &(parameters.create.payload)(&record, match parameters.get_matching_company_id_for_association {
                Some(get_matching_company_id_for_association) => match opt_comp {
                    Some(opt_comp) => Some(get_matching_company_id_for_association(opt_comp).await?),
                    None => None
                },
                None => None
            })?
        ).await?
    })
}

// Outdated:
// create_record: impl Fn(T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>
// update record takes in the record received, and the matching record in the other application
// update_record: impl Fn(T, T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>