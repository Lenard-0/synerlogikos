use crate::IntegrationRecord;

pub async fn get_record<T: IntegrationRecord>(
    id: &str,
    url: &str,
    properties: &str
) -> Result<T, String> {
    unimplemented!()
}