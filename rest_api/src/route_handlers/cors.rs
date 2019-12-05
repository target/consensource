#[options("/users")]
pub fn cors_users_route<'a>() -> &'a str {
    "Hello from CORS /api/users"
}

#[options("/users/authenticate")]
pub fn cors_users_auth_route<'a>() -> &'a str {
    "Hello from CORS /api/users/authenticate"
}

#[options("/batches")]
pub fn cors_batches_route<'a>() -> &'a str {
    "Hello from CORS /batches"
}
