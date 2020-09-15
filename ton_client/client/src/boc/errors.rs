/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::ApiError;
use std::fmt::Display;

pub enum Code {
    BocInvalidBoc = 200,
    BocSerializationError = 201,
}
pub struct Error;

fn error(code: Code, message: String) -> ApiError {
    ApiError::with_code_message(code as isize, message)
}

impl Error {
    pub fn boc_invalid_boc<E: Display>(err: E) -> ApiError {
        error(
            Code::BocInvalidBoc,
            format!("Invalid BOC: {}", err))
    }

    pub fn boc_serialization_error<E: Display>(err: E, name: &str) -> ApiError {
        error(
            Code::BocSerializationError,
            format!("Cannot serialize {}: {}", name, err))
    }
}
