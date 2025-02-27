use anyhow::Error as AnyhowError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use super::RouteErrorOutput;
use crate::RouteInternalErrorOutput;

/// This Rust module provides a standard error type for routes.
/// It encapsulates information about errors that occur while handling requests.
/// It includes a status code, error details, any extra data,
/// and a public error message.
///
/// It includes the means to output these as Json for the user.
///
/// The output will be in the form:
/// ```json
///     {
///         "error": "My public error message"
///     }
/// ```
///
/// Most of the time you will want to simply return one of:
///
///  - `RouteError::new_unauthorised()`
///  - `RouteError::new_not_found()`
///  - `RouteError::new_bad_request()`
///  - `RouteError::new_internal_server()`
///
/// Depending on which is the most appropriate.
///
pub struct RouteError<S = (), const EXPOSE_INTERNAL_ERROR: bool = false>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    status_code: StatusCode,
    error: Option<AnyhowError>,
    extra_data: Option<Box<S>>,
    public_error_message: Option<String>,
}

impl RouteError<()> {
    pub fn new_unauthorized() -> RouteError<()> {
        Self::new_from_status(StatusCode::UNAUTHORIZED)
    }

    pub fn new_not_found() -> RouteError<()> {
        Self::new_from_status(StatusCode::NOT_FOUND)
    }

    pub fn new_bad_request() -> RouteError<()> {
        Self::new_from_status(StatusCode::BAD_REQUEST)
    }

    pub fn new_internal_server() -> RouteError<()> {
        Self::new_from_status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn new_conflict() -> RouteError<()> {
        Self::new_from_status(StatusCode::CONFLICT)
    }

    pub fn new_forbidden() -> RouteError<()> {
        Self::new_from_status(StatusCode::FORBIDDEN)
    }

    pub fn new_from_status(status_code: StatusCode) -> RouteError<()> {
        Self {
            status_code,
            ..Self::default()
        }
    }
}

impl<S, const EXPOSE_INTERNAL_ERROR: bool> RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    /// Set a new status code for the error response.
    pub fn set_status_code(self, status_code: StatusCode) -> Self {
        Self {
            status_code,
            ..self
        }
    }

    /// Set an internal error.
    ///
    /// This is used for tracking the source of the error internally.
    pub fn set_error(self, error: AnyhowError) -> Self {
        Self {
            error: Some(error),
            ..self
        }
    }

    ///
    /// Sets additional error data to be added to the output.
    /// Data here must be serialisable into Json.
    ///
    /// # Example Code
    ///
    /// ```rust
    /// use axum_route_error::RouteError;
    /// use serde::Deserialize;
    /// use serde::Serialize;
    ///
    /// #[derive(Deserialize, Serialize, Debug)]
    /// pub struct UserErrorInformation {
    ///     pub guid: String
    /// }
    ///
    /// let guid = "abc123".to_string();
    /// let err = RouteError::new_not_found()
    ///     .set_error_data(UserErrorInformation {
    ///         guid,
    ///     });
    /// ```
    ///
    /// This will return a response with the JSON format:
    ///
    /// ```json
    /// {
    ///   "error": "The resource was not found",
    ///   "username": "<the-username>"
    /// }
    /// ```
    ///
    pub fn set_error_data<NewS>(self, extra_data: NewS) -> RouteError<NewS>
    where
        NewS: Serialize + for<'a> Deserialize<'a> + Debug,
    {
        RouteError {
            extra_data: Some(Box::new(extra_data)),
            status_code: self.status_code,
            error: self.error,
            public_error_message: self.public_error_message,
        }
    }

    /// Set the error message to display within the error.
    ///
    /// If this is not set, then an appropriate message is provided
    /// based on the status code.
    pub fn set_public_error_message(self, public_error_message: &str) -> Self {
        Self {
            public_error_message: Some(public_error_message.to_string()),
            ..self
        }
    }

    /// Returns the error message that will be shown to the end user.
    pub fn public_error_message<'a>(&'a self) -> &'a str {
        if let Some(public_error_message) = self.public_error_message.as_ref() {
            return public_error_message;
        }

        status_code_to_public_message(self.status_code())
    }

    /// Returns the status code for the response.
    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

impl<S, const EXPOSE_INTERNAL_ERROR: bool> Default for RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    fn default() -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error: None,
            extra_data: None,
            public_error_message: None,
        }
    }
}

impl<S, const EXPOSE_INTERNAL_ERROR: bool> IntoResponse for RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    fn into_response(self) -> Response {
        let status = self.status_code();
        let extra_data = self.extra_data;
        let error = match self.public_error_message {
            Some(public_error_message) => public_error_message,
            None => status_code_to_public_message(status).to_string(),
        };

        let internal_error = if EXPOSE_INTERNAL_ERROR {
            self.error.map(|err| RouteInternalErrorOutput {
                name: format!("{}", err),
                debug: format!("{:?}", err),
            })
        } else {
            None
        };

        let output = RouteErrorOutput {
            error,
            internal_error,
            extra_data,
            ..RouteErrorOutput::default()
        };
        let body = Json(output);

        (status, body).into_response()
    }
}

impl<S, const EXPOSE_INTERNAL_ERROR: bool> Debug for RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}, {:?}", self.public_error_message(), self.error)
    }
}

impl<S, const EXPOSE_INTERNAL_ERROR: bool> Display for RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.public_error_message())
    }
}

/// This essentially means if you can turn it into an Anyhow,
/// then you can turn it into a RouteError.
impl<S, const EXPOSE_INTERNAL_ERROR: bool, FE> From<FE> for RouteError<S, EXPOSE_INTERNAL_ERROR>
where
    S: Serialize + for<'a> Deserialize<'a> + Debug,
    FE: Into<AnyhowError>,
{
    fn from(error: FE) -> Self {
        let anyhow_error: AnyhowError = error.into();
        ::tracing::error!("{:?}", anyhow_error);

        RouteError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error: Some(anyhow_error),
            ..Self::default()
        }
    }
}

fn status_code_to_public_message(status_code: StatusCode) -> &'static str {
    match status_code {
        StatusCode::CONFLICT => "The request is not allowed",
        StatusCode::UNAUTHORIZED => "You are not authorised to access this endpoint",
        StatusCode::NOT_FOUND => "The resource was not found",
        StatusCode::BAD_REQUEST => "Bad request made",
        StatusCode::FORBIDDEN => "Request is forbidden",
        StatusCode::IM_A_TEAPOT => "I'm a teapot",
        StatusCode::TOO_MANY_REQUESTS => "Too many requests",
        StatusCode::BAD_GATEWAY => "Bad gateway",
        StatusCode::SERVICE_UNAVAILABLE => "Service unavailable",
        StatusCode::GATEWAY_TIMEOUT => "Gateway timeout",
        StatusCode::INTERNAL_SERVER_ERROR => "An unexpected error occurred",
        _ => "An unknown error occurred",
    }
}

#[cfg(test)]
mod test_route_error {
    use super::*;
    use crate::RouteErrorOutput;
    use anyhow::anyhow;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;
    use serde_json::from_slice;

    #[tokio::test]
    async fn it_should_not_output_internal_error() {
        fn raise_error() -> Result<(), RouteError> {
            Err(anyhow!("Too many foxes in the DB"))?;

            Ok(())
        }

        let err = raise_error().unwrap_err();
        let response = err.into_response();
        let response_body = response.into_body();
        let response_bytes = response_body.collect().await.unwrap().to_bytes();
        let body = from_slice::<RouteErrorOutput<()>>(&response_bytes).unwrap();

        assert_eq!(body.internal_error, None);
    }
}
