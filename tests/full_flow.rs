
mod tests {
    use reqwest::Error;
    use synerlogikos::{get::get_record, sync::sync_record, IntegrationRecord};

    #[tokio::test]
    async fn can_sync_contact() {
        let contact_id = "11111";

        sync_record(
            contact_id,
            get_hs_contact,
            meets_conditions,
            find_matching,
            create_record,
            update_record
        ).await.unwrap();

        struct Contact {}
        impl IntegrationRecord for Contact {}

        async fn get_hs_contact(record_id: &str) -> Result<Contact, Error> {
            return Contact.get(record_id, "hubapi.com", "").await
        }
    }
}
