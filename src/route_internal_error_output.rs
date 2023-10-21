use ::serde::Deserialize;
use ::serde::Serialize;
use ::std::fmt::Debug;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RouteInternalErrorOutput {
    pub name: String,
    pub debug: String,
}
