#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

//! This crate is meant to be used by webdevs to easily integrate into their projects
//! by running a microservice that allows them to run bdk to make `op_return` transactions in the bitcoin blockhchain

use doc_comment::doc_comment;

/// entry point for the webserver
fn main() {

}

doc_comment!(concat!("fooo", "or not foo"), pub struct Foo {});
