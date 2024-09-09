use std::fmt::Debug;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::IntegrationRecord;


pub async fn create_record<T>(
    create_url: &str,
    payload: &Value
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let client = Client::new();

    let response = client
        .post(create_url)
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