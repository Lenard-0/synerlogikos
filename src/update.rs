use std::fmt::Debug;
use serde::Deserialize;
use serde_json::Value;
use crate::{request::{requesting, HttpMethod}, sync::ConstructUrl, ApiClient, IntegrationRecord};

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
    let _response = requesting(
        &update_url(api_client, _type, &existing_id),
        Some(payload),
        HttpMethod::Patch,
        api_client
    ).await?;

    Ok(())
}