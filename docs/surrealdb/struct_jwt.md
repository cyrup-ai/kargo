# Struct `Jwt`

A JSON Web Token for authenticating with the server.
This struct represents a JSON Web Token (JWT) that can be used for authentication purposes.
It is important to note that this implementation provide some security measures to
protect the token:
* the debug implementation just prints `Jwt(REDACTED)`,
* `Display` is not implemented so you can't call `.to_string()` on it
You can still have access to the token string using either
[`as_insecure_token`](Jwt::as_insecure_token) or [`into_insecure_token`](Jwt::into_insecure_token) functions.
However, you should take care to ensure that only authorized users have access to the JWT.
For example:
* it can be stored in a secure cookie,
* stored in a database with restricted access,
* or encrypted in conjunction with other encryption mechanisms.

## Fields

Field information will be available in a future version.

