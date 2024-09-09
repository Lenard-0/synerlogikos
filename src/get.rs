use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use crate::IntegrationRecord;

pub async fn get_record<T: IntegrationRecord + for<'de> Deserialize<'de>>(
    final_url: &str,
    deserialize: Option<fn(&Value) -> T>,
) -> Result<T, String> {
    let client = Client::new();

    let response = client
        .get(final_url)
        .send()
        .await
        .map_err(|err| format!("Error sending get record request: {}     final_url: {}", err, final_url))?;

    if !response.status().is_success() {
        return Err(format!("Error sending get record record: received status code {}    final_url: {}", response.status(), final_url));
    }

    let record = match deserialize {
        Some(deserialize) => {
            let record: Value = response
                .json()
                .await
                .map_err(|err| format!("Error deserializing response: {}", err))?;
            deserialize(&record)
        },
        None => {
            response
                .json::<T>()
                .await
                .map_err(|err| format!("Error deserializing response: {}", err))?
        }
    };

    Ok(record)
}
