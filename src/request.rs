use reqwest::Client;
use serde_json::Value;
use crate::ApiClient;

pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

pub async fn requesting(
    url: &str,
    payload: Option<&Value>,
    method: HttpMethod,
    api_client: &Box<dyn ApiClient>,
) -> Result<Value, String> {
    let mut reqwest_builder = match method {
        HttpMethod::Get => Client::new().get(url),
        HttpMethod::Post => Client::new().post(url),
        HttpMethod::Put => Client::new().put(url),
        HttpMethod::Patch => Client::new().patch(url),
        HttpMethod::Delete => Client::new().delete(url),
    };

    for header in api_client.headers() {
        reqwest_builder = reqwest_builder.header(header.0, header.1);
    }

    let req = reqwest_builder
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("Error sending request: {:#?}", err))?;

    let status = req.status();
    if !status.is_success() {
        let response_text = req
        .text()
        .await
        .unwrap_or_else(|_| "Failed to read response body".to_string());

        return Err(format!(
            "Error: received status code {}\nurl: {}\npayload: {:#?}\nresponse body: {}",
            status, url, payload, response_text
        ));
    }

    let res: Value = req
        .json()
        .await
        .map_err(|err| format!("Error deserializing response: {:#?}", err))?;

    Ok(res)
}