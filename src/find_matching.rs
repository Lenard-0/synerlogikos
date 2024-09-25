
use std::{fmt::Debug, thread::sleep, time::Duration};

use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use crate::{ApiClient, IntegrationRecord};


pub async fn find_matching<T, C: ApiClient>(
    record: &T,
    client: &C,
    properties: Vec<String>,
    construct_search_url: fn(property: &str, value: &str) -> Result<String, String>,
    payload: Option<fn(property: &str, value: &str) -> Value>,
    index_array: fn(json: Value) -> Value,
) -> Result<Option<Value>, String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    for property in properties {
        let found_matching: Option<Value> = search_by_property(
            &property,
            record,
            construct_search_url,
            payload,
            index_array,
            &client.access_token()
        ).await?;

        if found_matching.is_some() {
            return Ok(found_matching)
        }

        sleep(Duration::from_millis(500));
    }

    return Ok(None)
}

async fn search_by_property<T>(
    property: &str,
    record: &T,
    construct_search_url: fn(property: &str, value: &str) -> Result<String, String>,
    payload: Option<fn(property: &str, value: &str) -> Value>,
    index_array: fn(json: Value) -> Value,
    token: &str
) -> Result<Option<Value>, String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let client = Client::new();

    let property_value = match record.index_property(property) {
        Some(value) => value,
        None => {
            println!("Potential error! Could not index property: {property} on record: {:#?}", record);
            return Ok(None)
        }
    };
    return match payload {
        Some(payload) => match client.post(construct_search_url(&property, &property_value)?)
            .bearer_auth(&token)
            .json(&payload(&property, &property_value))
            .send()
            .await {
            Ok(res) => check_array_search_only_contains_one(res, index_array).await,
            Err(err) => Err(format!("Error searching for matching: {}", err))
        },
        None => match client.get(construct_search_url(&property, &property_value)?)
            .bearer_auth(&token)
            .send()
            .await {
            Ok(res) => check_array_search_only_contains_one(res, index_array).await,
            Err(err) => Err(format!("Error searching for matching: {}", err))
        }
    }
}

async fn check_array_search_only_contains_one(
    res: Response,
    index_array: fn(json: Value) -> Value,
) -> Result<Option<Value>, String> {
    let json: Value = res.json().await.expect("Find match res not json");
    match index_array(json).as_array() {
        Some(arr) if arr.len() == 1 => Ok(Some(arr[0].clone())),
        _ => Ok(None),
    }
}