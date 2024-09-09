use std::fmt::Debug;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::{sync::ConstructUrl, ApiClient, IntegrationRecord};


pub async fn create_record<T, C: ApiClient>(
    construct_url: ConstructUrl<C>,
    api_client: &C,
    _type: &str,
    existing_id: &Option<String>,
    payload: &Value,
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let reqwest_client = Client::new();
    let create_url = construct_url(api_client, _type, existing_id);

    let response = reqwest_client
        .post(&create_url)
        .json(payload)
        .send()
        .await
        .map_err(|err| format!("Error sending create request: {}     create_url: {} payload: {:#?}", err, create_url, payload))?;

    if !response.status().is_success() {
        return Err(format!("Error creating record: received status code {}    create_url: {}   payload: {:#?}", response.status(), create_url, payload));
    }

    // Should impl properly at some point
    let create_res = response
        .json()
        .await
        .map_err(|err| format!("Error deserializing response: {}", err))?;

    return Ok(())
}