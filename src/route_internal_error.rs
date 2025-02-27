use super::RouteError;

/// This is for **exposing internal errors publically.**
/// It is desirable for internal services, where you do want to expose
/// what has gone wrong as a part of the return.
pub type RouteInternalError<S = ()> = RouteError<S, true>;

#[cfg(test)]
mod test_route_internal_error {
    use super::*;
    use crate::RouteErrorOutput;
    use anyhow::anyhow;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;
    use serde_json::from_slice;

    #[tokio::test]
    async fn it_should_output_internal_error() {
        fn raise_error() -> Result<(), RouteInternalError> {
            Err(anyhow!("Too many foxes in the DB"))?;

            Ok(())
        }

        let err = raise_error().unwrap_err();
        let response = err.into_response();
        let response_body = response.into_body();
        let response_bytes = response_body.collect().await.unwrap().to_bytes();
        let body = from_slice::<RouteErrorOutput<()>>(&response_bytes).unwrap();

        assert_eq!(
            body.internal_error.unwrap().name,
            "Too many foxes in the DB"
        );
    }
}
