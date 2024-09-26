
pub mod sync;
pub mod get;
pub mod create;
pub mod update;
pub mod find_matching;
pub mod request;

pub trait IntegrationRecord {
    fn index_property(&self, property: &str) -> Option<String>;
    fn _type(&self) -> String;
}

pub trait ApiClient {
    fn headers(&self) -> Vec<(&str, &str)>;
    fn clone_box(&self) -> Box<dyn ApiClient>;
    fn account_id(&self) -> String;
}
