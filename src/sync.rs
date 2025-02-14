

use serde::Deserialize;
use serde_json::Value;

use crate::{create::create_record, find_matching::{find_matching, find_matching_associate_record}, get::get_record, update::update_record, ApiClient, IntegrationRecord};
use core::fmt::Debug;
use std::{future::Future, pin::Pin};

pub struct SyncRecordData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub get: GetData,
    pub create: CreateData<T>,
    pub update: UpdateData<T>,
    pub find_matching: FindMatchingData,
    pub index_matching_id: fn(json: &Value) -> Result<String, String>,
    pub deserialize: Option<fn(&Value) -> T>,
    pub from_api_client: Box<dyn ApiClient>,
    pub to_api_client: Box<dyn ApiClient>,
    pub to_type: String,
    pub get_matching_record_id_for_association: Option<AssociateRecord>,
}

pub struct AssociateRecord {
    pub find_matching: FindMatchingData,
    pub to_record: Box<dyn Fn(&Value) -> Box<dyn IntegrationRecord>>,
    pub extract_id: fn(&Value) -> Result<Option<String>, String>,
}

pub struct GetData {
    pub url: String,
}

pub struct CreateData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: ConstructUrl,
    pub payload: ConstructPayload<T>
}

pub type ConstructPayload<T> = fn(&T, Option<String>) -> Result<Value, String>;

pub type ConstructUrl = fn(client: &Box<dyn ApiClient>, _type: &str, existing_id: &Option<String>) -> String;

pub struct UpdateData<T> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    pub url: ConstructUrl,
    pub payload: ConstructPayload<T>
}

pub struct FindMatchingData {
    pub properties: Vec<String>,
    pub construct_search_url: fn(obj_type: &str, property: &str, value: &str) -> Result<String, String>,
    pub payload: Option<fn(property: &str, value: &str) -> Value>,
    pub index_array: fn(json: Value) -> Value,
}


/// To sync a record from one application to another
/// Both records should implement IntegrationRecord
/// Intended use is right after receiving a webhook of a change, pass the ID and the relevant functions here to sync
pub async fn sync_record<T>(
    parameters: SyncRecordData<T>,
    meets_conditions: Option<impl Fn(T, Value, Box<dyn ApiClient>) -> Pin<Box<dyn Future<Output = Result<Option<Option<Value>>, String>>>>>
) -> Result<(), String> where T: IntegrationRecord + Clone + Debug + for<'de> Deserialize<'de> {
    let (record, json) = get_record(
        &parameters.get.url,
        parameters.deserialize.clone(),
        &parameters.from_api_client
    ).await?;
    println!("got record: {:#?}", record);

    return match meets_conditions {
        Some(meets_conditions) => match meets_conditions(
            record.clone(),
            json,
            parameters.from_api_client.clone_box()
        ).await? {
            Some(opt_company) => actualise_sync(parameters, record, opt_company).await,
            None => {
                println!("Record did not meet conditions to sync");
                return Ok(())
            }
        },
        None => actualise_sync(parameters, record, None).await
    }
}

async fn actualise_sync<T>(
    parameters: SyncRecordData<T>,
    record: T,
    opt_comp: Option<Value>
) -> Result<(), String> where T: IntegrationRecord + Clone + Debug + for<'de> Deserialize<'de> {

    Ok(match find_matching(
        Box::new(record.clone()),
        &parameters.to_api_client,
        parameters.find_matching.properties,
        parameters.find_matching.construct_search_url,
        parameters.find_matching.payload,
        parameters.find_matching.index_array,
    ).await? {
        Some(matching_record) => update_record::<T>(
            parameters.update.url,
            &parameters.to_api_client,
            &parameters.to_type,
            Some((parameters.index_matching_id)(&matching_record)?),
            &(parameters.create.payload)(&record, find_matching_associate_record(
                opt_comp,
                &parameters.to_api_client,
                parameters.get_matching_record_id_for_association
            ).await?)?
        ).await?,
        None => create_record::<T>(
            parameters.create.url,
            &parameters.to_api_client,
            &parameters.to_type,
            None,
            &(parameters.create.payload)(&record, find_matching_associate_record(
                opt_comp,
                &parameters.to_api_client,
                parameters.get_matching_record_id_for_association
            ).await?)?
        ).await?
    })
}