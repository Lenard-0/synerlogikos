
use std::{future::Future, pin::Pin};

use crate::IntegrationRecord;


/// To sync a record from one application to another
/// Both records should implement IntegrationRecord
/// Intended use is right after receiving a webhook of a change, pass the ID and the relevant functions here to sync
pub async fn sync_record<T: IntegrationRecord>(
    record_id: &str,
    get_record: impl Fn(&str) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (id: &str) -> Result<Option<T>, String>
    meets_conditions: fn(record: &T) -> bool,
    // find matching should return the matching record from the other system
    // both records should implement integration record
    find_matching: impl Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<T>, String>>>>, // async fn (record: T) -> Result<Option<T>, String>
    create_record: impl Fn(T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>
    // update record takes in the record received, and the matching record in the other application
    update_record: impl Fn(T, T) -> Pin<Box<dyn Future<Output = Result<(), String>>>>, // async fn (record: T) -> Result<(), String>
) -> Result<(), String> {
    let record: T = match get_record(&record_id).await? {
        Some(record) => record,
        None => return Err(format!("No record found when trying to query record from ID at start of record sync!: {record_id}"))
    };

    if meets_conditions(&record) {
        match find_matching(&record).await? {
            Some(matching_record) => update_record(record, matching_record).await?,
            None => create_record(record).await?
        }
    }

    return Ok(())
}