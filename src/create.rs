use crate::IntegrationRecord;



pub async fn create_record<T: IntegrationRecord>(
    record: T
) -> Result<(), String> {
    unimplemented!()
}