
use std::fmt::Debug;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::IntegrationRecord;


pub async fn find_matching<T>(
    properties: Vec<String>,
    construct_search_url: fn(property: &str) -> String,
    payload: fn(property: &str) -> Value,
    index_array: fn(json: Value) -> Value,
    token: &str
) -> Result<Option<Value>, String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    for property in properties {
        let found_matching: Option<Value> = search_by_property(&property, construct_search_url, payload, index_array, token).await?;

        if found_matching.is_some() {
            return Ok(found_matching)
        }
    }

    return Ok(None)
}

async fn search_by_property(
    property: &str,
    construct_search_url: fn(property: &str) -> String,
    payload: fn(property: &str) -> Value,
    index_array: fn(json: Value) -> Value,
    token: &str
) -> Result<Option<Value>, String> {
    let client = Client::new();
    match client.post(construct_search_url(&property))
        .bearer_auth(&token)
        .json(&payload(&property))
        .send()
        .await
            {
            Ok(res) => {
                let json: Value = res.json().await.expect("Find match res not json");
                match index_array(json).as_array() {
                    Some(arr) if arr.len() == 1 => Ok(Some(arr[0].clone())),
                    _ => Ok(None)
                }
            },
            Err(err) => Err(format!("Error searching for matching: {}", err))
    }
}