
pub mod sync;
pub mod get;
pub mod create;
pub mod update;
pub mod find_matching;

pub trait IntegrationRecord {
    fn index_property(&self, property: &str) -> Option<String>;
}

pub trait ApiClient {
    fn access_token(&self) -> String;
    fn clone_box(&self) -> Box<dyn ApiClient>;
}
