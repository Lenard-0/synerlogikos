use std::fmt::Debug;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::IntegrationRecord;


pub async fn update_record<T>(
    update_url: &str,
    payload: &Value
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let client = Client::new();

    let response = client
        .patch(update_url)
        .json(payload)
        .send()
        .await
        .map_err(|err| format!("Error sending update request: {}     update_url: {} payload: {:#?}", err, update_url, payload))?;

    if !response.status().is_success() {
        return Err(format!("Error updating record: received status code {}    update_url: {}   payload: {:#?}", response.status(), update_url, payload));
    }

    // Should impl properly at some point
    let update_res = response
        .json()
        .await
        .map_err(|err| format!("Error deserializing response: {}", err))?;

    return Ok(())
}