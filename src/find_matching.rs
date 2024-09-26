
use std::{thread::sleep, time::Duration};
use serde_json::Value;
use crate::{request::{requesting, HttpMethod}, sync::AssociateRecord, ApiClient, IntegrationRecord};

pub async fn find_matching(
    record: Box<impl IntegrationRecord + ? Sized>,
    client: &Box<dyn ApiClient>,
    properties: Vec<String>,
    construct_search_url: fn(obj_type: &str, property: &str, value: &str) -> Result<String, String>,
    payload: Option<fn(property: &str, value: &str) -> Value>,
    index_array: fn(json: Value) -> Value,
) -> Result<Option<Value>, String> {
    for property in properties {
        let found_matching: Option<Value> = search_by_property(
            &property,
            &record,
            construct_search_url,
            payload,
            index_array,
            client
        ).await?;

        if found_matching.is_some() {
            return Ok(found_matching)
        }

        sleep(Duration::from_millis(500));
    }

    return Ok(None)
}

async fn search_by_property(
    property: &str,
    record: &Box<impl IntegrationRecord + ? Sized>,
    construct_search_url: fn(obj_type: &str, property: &str, value: &str) -> Result<String, String>,
    payload: Option<fn(property: &str, value: &str) -> Value>,
    index_array: fn(json: Value) -> Value,
    api_client: &Box<dyn ApiClient>
) -> Result<Option<Value>, String> {
    let property_value = match record.index_property(property) {
        Some(value) => value,
        None => {
            println!("Potential error! Could not index property: {property} on record");
            return Ok(None)
        }
    };

    return match payload {
        Some(payload) => check_array_search_only_contains_one(requesting(
            &construct_search_url(&record._type(), &property, &property_value)?,
            Some(&payload(&property, &property_value)),
            HttpMethod::Post,
            api_client
        ).await?, index_array).await,
        None => check_array_search_only_contains_one(requesting(
            &construct_search_url(&record._type(), &property, &property_value)?,
            None,
            HttpMethod::Get,
            api_client
        ).await?, index_array).await
    }
}

async fn check_array_search_only_contains_one(
    json: Value,
    index_array: fn(json: Value) -> Value,
) -> Result<Option<Value>, String> {
    match index_array(json).as_array() {
        Some(arr) if arr.len() == 1 => Ok(Some(arr[0].clone())),
        _ => Ok(None),
    }
}

pub async fn find_matching_associate_record(
    from_record: Option<Value>,
    api_client: &Box<dyn ApiClient>,
    get_matching_record_id_for_association: Option<AssociateRecord>,
) -> Result<Option<String>, String> {
    match from_record {
        Some(record) => match get_matching_record_id_for_association {
            Some(get_matching_record_id_for_association) => {
                let matching = find_matching(
                    (*get_matching_record_id_for_association.to_record)(&record),
                    api_client,
                    get_matching_record_id_for_association.find_matching.properties,
                    get_matching_record_id_for_association.find_matching.construct_search_url,
                    get_matching_record_id_for_association.find_matching.payload,
                    get_matching_record_id_for_association.find_matching.index_array,
                ).await?;
                match matching {
                    Some(matching) => Ok((get_matching_record_id_for_association.extract_id)(&matching)?),
                    None => Ok(None)
                }
            },
            None => Ok(None)
        }
        None => Ok(None)
    }
}