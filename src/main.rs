#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

//! This crate is meant to be used by webdevs to easily integrate into their projects
//! by running a microservice that allows them to run bdk to make `op_return` transactions in the bitcoin blockhchain

mod error;

use doc_comment::doc_comment;
use std::fmt;
use std::io::IsTerminal;

use crate::error::GraffitiError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
/// entry point for the webserver
use axum::{extract::Path, routing::get, Json, Router};
use better_panic::Settings;
use serde_json::json;
use tracing::{error_span, info, instrument};
use tracing_subscriber::layer::SubscriberExt;

#[tracing::instrument]
async fn get_op_return(Path(data): Path<String>) -> error::Result<impl IntoResponse> {
    info!("Received GET request with data: {}", data);

    let j = json!({ "address": data });

    Ok(Json(j))
}

#[tracing::instrument]
async fn write_op_return(Path(data): Path<String>) -> error::Result<impl IntoResponse> {
    info!("Received WRITE request with data: {}", data);
    let j = json!({ "address": data });

    Ok(Json(j))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Settings::debug()
        .most_recent_first(false)
        .lineno_suffix(true)
        .install();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let app = Router::new()
        .route("/get_op_return/:data", get(get_op_return))
        .route("/write_op_return/:data", get(write_op_return));

    // let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await.unwrap();
    let addr = if cfg!(debug_assertions) {
        std::net::SocketAddr::from(([127, 0, 0, 1], 9000))
    } else {
        std::net::SocketAddr::from(([0, 0, 0, 0], 9000))
    };
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server running on {:?}", listener);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

doc_comment!(concat!("fooo", "or not foo"), pub struct Foo {});
