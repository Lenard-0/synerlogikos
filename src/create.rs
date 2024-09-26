use std::fmt::Debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::{sync::ConstructUrl, ApiClient, IntegrationRecord};

pub async fn create_record<T>(
    construct_url: ConstructUrl,
    api_client: &Box<dyn ApiClient>,
    _type: &str,
    existing_id: Option<String>,
    payload: &Value,
) -> Result<(), String>
where
    T: IntegrationRecord + Debug + for<'de> Deserialize<'de>,
{
    let reqwest_client = Client::new();
    let create_url = construct_url(api_client, _type, &existing_id);

    let response = reqwest_client
        .post(&create_url)
        .bearer_auth(api_client.access_token())
        .json(payload)
        .send()
        .await
        .map_err(|err| {
            format!(
                "Error sending create request: {:#?}\ncreate_url: {}\npayload: {:#?}",
                err, create_url, payload
            )
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body".to_string());

        return Err(format!(
            "Error creating record: received status code {}\ncreate_url: {}\npayload: {:#?}\nresponse body: {}",
            status, create_url, payload, response_text
        ));
    }

    let create_res: Value = response
        .json()
        .await
        .map_err(|err| format!("Error deserializing response: {:#?}", err))?;

    Ok(())
}
