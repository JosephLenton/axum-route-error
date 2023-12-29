<div align="center">
  <h1>
    Axum Route Error
  </h1>

  <h3>
    A common error response for Axum routes
  </h3>

  [![crate](https://img.shields.io/crates/v/axum-route-error.svg)](https://crates.io/crates/axum-route-error)
  [![docs](https://docs.rs/axum-route-error/badge.svg)](https://docs.rs/axum-route-error)

  <br>
</div>

This exists to encapsulate a number of common patterns when returning errors from Axum routes.
These patterns include:

 - wanting to return error responses as a JSON object
 - to automatically convert errors raised in the route hander into that standardised JSON response
 - provide appropriate default error messages for public use, which takes into account the status code in use
 - a need to optionally add extra data to the error JSON output

## A basic tutorial

Below is a mock example route to download a User object:

```rust
use ::axum::extract::State;
use ::axum_route_error::RouteError;
use ::sea_orm::DatabaseConnection;

pub async fn route_get_user(
    State(ref db): State<DatabaseConnection>,
    Path(username): Path<String>
) -> Result<ExampleUser, RouteError> {
    // If this errors, then a `RouteError::new_internal_server()` is returned.
    let user = get_user_from_database(db, &username).await?;

    Ok(user)
}
```

If the `get_user_from_database` function returns an error,
then the handler will return a Response.
The Response will have a 500 status code (for an internal error),
and return the following output:

```json
{
  "error": "An unexpected error occurred"
}
```

### Different `RouteError` types

Let's presume `get_user_from_database` returns a `Result<Option, Error>`.
We want to return a 500 if it returns an error (like above),
and change the code to return a 404 if the user is not found.

```rust
use ::axum::extract::State;
use ::axum_route_error::RouteError;
use ::sea_orm::DatabaseConnection;

pub async fn route_get_user(
    State(ref db): State<DatabaseConnection>,
    Path(username): Path<String>
) -> Result<ExampleUser, RouteError> {
    let user = get_user_from_database(db, &username).await?
      // This additional line will return a 404 if the user is not found.
      .ok_or_else(|| RouteError::new_not_found())?;

    Ok(user)
}
```

If the user is not found (`get_user_from_database` returns `None`),
then this will return a 404 Response with the following JSON:

```json
{
  "error": "The resource was not found"
}
```

## Adding additional error data

Next let's add extra information to the error.
Something more than just an error message.

This can be done by making a new type that serializes using Serde,
and then adding this to the `RouteError`.

```rust
use ::axum::extract::State;
use ::axum_route_error::RouteError;
use ::sea_orm::DatabaseConnection;
use ::serde::Deserialize;
use ::serde::Serialize;

// The additional error information needs to derive these three traits.
#[derive(Deserialize, Serialize, Debug)]
pub struct UserErrorInformation {
  pub username: String
}

pub async fn route_get_user(
    State(ref db): State<DatabaseConnection>,
    Path(username): Path<String>
// The `RouteError` needs the additional data marked here
) -> Result<ExampleUser, RouteError<UserErrorInformation>> {
    let user = get_user_from_database(db, &username).await?
      .ok_or_else(move || {
        // Then you can add the data through method chaining
        RouteError::new_not_found()
          .set_error_data(UserErrorInformation {
            username,
          })
      })?;

    Ok(user)
}
```

If the user is not found (`get_user_from_database` returns `None`),
then this will return a 404 Response with the following JSON:

```json
{
  "error": "The resource was not found",
  "username": "<the-username>"
}
```

## Making Internal Errors public

Sometimes you *want* to make internal errors public,
such as for internal services.

For this you can use the `RouteInternalError`. It's identical,
but adds adds `internal_error` information to the response.
