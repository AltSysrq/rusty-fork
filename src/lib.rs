//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

extern crate fnv;
#[macro_use] extern crate quick_error;
extern crate tempfile;

#[macro_use] mod sugar;
mod error;
mod cmdline;
mod fork;

pub use sugar::ForkTestId;
pub use error::{Error, Result};
pub use fork::fork;
