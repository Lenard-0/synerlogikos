use serde::Deserialize;
use serde_json::Value;
use crate::{request::{requesting, HttpMethod}, ApiClient, IntegrationRecord};

pub async fn get_record<T: IntegrationRecord + for<'de> Deserialize<'de>>(
    final_url: &str,
    deserialize: Option<fn(&Value) -> T>,
    api_client: &Box<dyn ApiClient>
) -> Result<(T, Value), String> {
    let json: Value = requesting(
        final_url,
        None,
        HttpMethod::Get,
        api_client
    ).await?;

    let record = match deserialize {
        Some(deserialize) => deserialize(&json),
        None => serde_json::from_value(json).map_err(|err| format!("Error deserializing response: {:#?}", err))?
    };

    return Ok((record, Value::Null))
}
