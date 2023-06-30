use ::serde::Deserialize;
use ::serde::Serialize;
use ::std::fmt::Debug;

#[derive(Debug, Deserialize, Serialize)]
pub struct RouteErrorOutput<S> {
    pub error: String,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<S>,
}
