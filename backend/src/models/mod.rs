mod data;
mod response;
mod paginable;
mod user;
mod token_claims;
mod catalog;
mod tag_list;
mod manifest_info;
mod registry_client;
mod repository_info;
mod manifest_v2;
mod config_descriptor;
mod layer_descriptor;
mod tag_detail;

pub type Error = Box<dyn std::error::Error>;
pub use paginable::Paginable;
pub use registry_client::RegistryClient;
pub use token_claims::TokenClaims;

pub use user::User;

pub use response::{
    ApiResponse,
    CustomResponse,
    EmptyResponse,
    PagedResponse,
    Pagination,
};

pub struct AppState {
    pub secret: String,
    pub static_dir: String,
    pub user: User,
    pub registry_client: RegistryClient,
}
