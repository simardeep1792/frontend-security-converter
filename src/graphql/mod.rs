pub mod log_in_mutation;
pub mod authority;
pub mod user;
pub mod conversion_request;
pub mod metadata;

pub fn with_optional_bearer(
    request: reqwest::RequestBuilder,
    bearer: &str,
) -> reqwest::RequestBuilder {
    let bearer = bearer.trim();

    if bearer.is_empty() {
        request
    } else {
        request.header("Authorization", format!("Bearer {}", bearer))
    }
}

pub use log_in_mutation::*;
pub use authority::*;
pub use user::*;
pub use conversion_request::*;
pub use metadata::*;
