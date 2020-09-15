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

extern crate api_doc;
#[macro_use]
extern crate api_doc_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod encoding;
mod error;
mod dispatch;
pub mod client;
mod contracts;
mod crypto;
mod tvm;
mod boc;

#[cfg(feature = "node_interaction")]
pub mod queries;

mod interop;

#[cfg(test)]
mod tests;


pub use self::interop::*;

