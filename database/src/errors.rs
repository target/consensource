/*
 * Copyright 2018 Bitwise IO
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ------------------------------------------------------------------------------
 */

use std;
extern crate diesel;
extern crate r2d2;

#[derive(Debug)]
pub enum DatabaseError {
    ConnError(r2d2::Error),
    TransactionError(diesel::result::Error),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DatabaseError::ConnError(ref err) => write!(f, "Connection error: {}", err),
            DatabaseError::TransactionError(ref err) => write!(f, "Transaction error: {}", err),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn description(&self) -> &str {
        match *self {
            DatabaseError::ConnError(ref err) => err.description(),
            DatabaseError::TransactionError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            DatabaseError::ConnError(ref err) => Some(err),
            DatabaseError::TransactionError(ref err) => Some(err),
        }
    }
}

impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> String {
        match err {
            DatabaseError::ConnError(ref err) => format!("Connection error: {}", err),
            DatabaseError::TransactionError(ref err) => format!("Transaction error: {}", err),
        }
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(err: r2d2::Error) -> DatabaseError {
        DatabaseError::ConnError(err)
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> DatabaseError {
        DatabaseError::TransactionError(err)
    }
}
