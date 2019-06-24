use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket_contrib::json::JsonValue;
use std::io::Cursor;

#[catch(404)]
pub fn not_found() -> JsonValue {
    json!({
        "error": {
            "status": Status::NotFound.code,
            "message": "Not found"
        }
    })
}

#[catch(500)]
pub fn internal_error() -> JsonValue {
    json!({
        "error": {
            "status": Status::InternalServerError.code,
            "message": "Internal error"
        }
    })
}

#[catch(503)]
pub fn service_unavailable() -> JsonValue {
    json!({
        "error": {
            "status": Status::ServiceUnavailable.code,
            "message": "Service unavailable"
        }
    })
}

#[derive(Debug)]
pub enum ApiError {
    /// Defines the HTTP Errors that the API can return.
    BadRequest(String),
    InternalError(String),
    NotFound(String),
    TooManyRequests(String),
    ServiceUnavailable,
    Unauthorized,
}

impl<'r> Responder<'r> for ApiError {
    /// JSON responder for ApiErrors.
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        match self {
            ApiError::BadRequest(ref msg) => Response::build()
                .header(ContentType::JSON)
                .status(Status::BadRequest)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::BadRequest.code,
                            "message": format!("Bad request: {}", msg),
                        }
                    })
                    .to_string(),
                ))
                .ok(),
            ApiError::InternalError(ref msg) => Response::build()
                .header(ContentType::JSON)
                .status(Status::InternalServerError)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::InternalServerError.code,
                            "message": format!("Internal error: {}", msg),
                        }
                    })
                    .to_string(),
                ))
                .ok(),
            ApiError::NotFound(ref msg) => Response::build()
                .header(ContentType::JSON)
                .status(Status::NotFound)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::NotFound.code,
                            "message": format!("Not found: {}", msg),
                        }
                    })
                    .to_string(),
                ))
                .ok(),
            ApiError::TooManyRequests(ref msg) => Response::build()
                .header(ContentType::JSON)
                .status(Status::TooManyRequests)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::TooManyRequests.code,
                            "message": format!("Too many requests: {}", msg),
                        }
                    })
                    .to_string(),
                ))
                .ok(),
            ApiError::ServiceUnavailable => Response::build()
                .header(ContentType::JSON)
                .status(Status::ServiceUnavailable)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::ServiceUnavailable.code,
                            "message": "Service Unavailable",
                        }
                    })
                    .to_string(),
                ))
                .ok(),
            ApiError::Unauthorized => Response::build()
                .header(ContentType::JSON)
                .status(Status::Unauthorized)
                .sized_body(Cursor::new(
                    json!({
                        "error": {
                            "status": Status::Unauthorized.code,
                            "message": "Unauthorized!",
                        }
                    })
                    .to_string(),
                ))
                .ok(),
        }
    }
}

impl From<::diesel::result::Error> for ApiError {
    fn from(err: ::diesel::result::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}
