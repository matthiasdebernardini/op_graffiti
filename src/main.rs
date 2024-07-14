#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

//! This crate provides a microservice for web developers to easily integrate Bitcoin
//! `OP_RETURN` transaction capabilities into their projects.
//! It leverages the Bitcoin Development Kit (BDK) to offer a simple and efficient way
//! to create and manage `OP_RETURN` transactions on the Bitcoin network.

mod error;
mod routes;
mod tests;
mod util;

use crate::util::{setup_better_panic, setup_server, setup_tracer};
use axum::serve;
use tracing::info;

const EXTERNAL_DESCRIPTOR: &str = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/0/*)";
const INTERNAL_DESCRIPTOR: &str = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/1/*)";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_better_panic();

    setup_tracer();

    let (app, listener) = setup_server().await?;

    info!("Server running on {:?}", listener);

    serve(listener, app).await?;

    Ok(())
}
