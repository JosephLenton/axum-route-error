use ::serde::Deserialize;
use ::serde::Serialize;
use ::std::fmt::Debug;

use crate::RouteInternalErrorOutput;

#[derive(Debug, Deserialize, Serialize)]
pub struct RouteErrorOutput<S> {
    pub error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_error: Option<RouteInternalErrorOutput>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<S>,
}

impl<S> Default for RouteErrorOutput<S> {
    fn default() -> Self {
        Self {
            error: "An unknown error occurred".to_string(),
            internal_error: None,
            extra_data: None,
        }
    }
}
