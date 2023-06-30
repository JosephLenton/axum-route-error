<div align="center">
  <h1>
    Axum Route Error
  </h1>

  <h3>
    A common error response for Axum servers
  </h3>

  <br>
</div>

This exists to encapsulate a number of common patterns when returning errors from Axum routes.

These patterns include ...

 - returning errors as JSON objects
 - optionally adding data to those error objects
 - automatically converting errors raised in the route hander into a standardised error response
 - to hold 'public' error messages for the user, and internal errors
