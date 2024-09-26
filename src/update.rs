use std::fmt::Debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::{sync::ConstructUrl, ApiClient, IntegrationRecord};

pub async fn update_record<T>(
    update_url: ConstructUrl,
    api_client: &Box<dyn ApiClient>,
    _type: &str,
    existing_id: Option<String>,
    payload: &Value,
) -> Result<(), String>
where
    T: IntegrationRecord + Debug + for<'de> Deserialize<'de>,
{
    let reqwest_client = Client::new();

    let update_url = update_url(api_client, _type, &existing_id);

    // Enhanced error handling when sending the request
    let response = reqwest_client
        .patch(&update_url)
        .bearer_auth(api_client.access_token())
        .json(payload)
        .send()
        .await
        .map_err(|err| {
            format!(
                "Error sending update request: {:#?}\nupdate_url: {}\npayload: {:#?}",
                err, update_url, payload
            )
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body".to_string());

        return Err(format!(
            "Error updating record: received status code {}\nupdate_url: {}\npayload: {:#?}\nresponse body: {}",
            status, update_url, payload, response_text
        ));
    }

    // Enhanced error handling when deserializing the response
    let update_res: Value = response
        .json()
        .await
        .map_err(|err| format!("Error deserializing response: {:#?}", err))?;

    Ok(())
}
