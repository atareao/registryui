mod data;
mod response;
mod paginable;
mod user;
mod token_claims;
mod catalog;
mod tag_list;
mod manifest_info;
mod registry_client;

pub type Error = Box<dyn std::error::Error>;
pub use paginable::Paginable;
pub use token_claims::TokenClaims;

pub use user::User;

pub use data::Data;
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
}
