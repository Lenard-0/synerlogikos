use std::fmt::Debug;
use serde::Deserialize;
use serde_json::Value;
use crate::{request::{requesting, HttpMethod}, sync::ConstructUrl, ApiClient, IntegrationRecord};

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
    let _response = requesting(
        &construct_url(api_client, _type, &existing_id),
        Some(payload),
        HttpMethod::Post,
        api_client
    ).await?;

    Ok(())
}
