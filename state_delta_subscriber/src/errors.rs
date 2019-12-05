use database::errors::DatabaseError;
use std;

#[derive(Debug)]
pub enum SubscriberError {
    ConnError(String),
    EventParseError(String),
    DBError(DatabaseError),
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SubscriberError::ConnError(ref err) => {
                write!(f, "Error connecting to validator {}", err)
            }
            SubscriberError::EventParseError(ref err) => write!(f, "Error parsing event {}", err),
            SubscriberError::DBError(ref err) => {
                write!(f, "The database returned an error {}", err)
            }
        }
    }
}

impl std::error::Error for SubscriberError {
    fn description(&self) -> &str {
        match *self {
            SubscriberError::ConnError(ref err) => err,
            SubscriberError::EventParseError(ref err) => err,
            SubscriberError::DBError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            SubscriberError::ConnError(_) => None,
            SubscriberError::EventParseError(_) => None,
            SubscriberError::DBError(ref err) => Some(err),
        }
    }
}

impl From<SubscriberError> for String {
    fn from(err: SubscriberError) -> String {
        match err {
            SubscriberError::ConnError(ref err) => format!("Error connecting to validator {}", err),
            SubscriberError::EventParseError(ref err) => format!("Error parsing event {}", err),
            SubscriberError::DBError(ref err) => format!("Error parsing event {}", err),
        }
    }
}

impl From<DatabaseError> for SubscriberError {
    fn from(err: DatabaseError) -> SubscriberError {
        SubscriberError::DBError(err)
    }
}
