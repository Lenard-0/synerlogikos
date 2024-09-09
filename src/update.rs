use std::fmt::Debug;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::{sync::ConstructUrl, ApiClient, IntegrationRecord};


pub async fn update_record<T, C: ApiClient>(
    update_url: ConstructUrl<C>,
    api_client: &C,
    _type: &str,
    existing_id: Option<String>,
    payload: &Value
) -> Result<(), String> where T: IntegrationRecord + Debug + for<'de> Deserialize<'de> {
    let reqwest_client = Client::new();

    let update_url = update_url(api_client, _type, &existing_id);
    let response = reqwest_client
        .patch(&update_url)
        .bearer_auth(api_client.access_token())
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