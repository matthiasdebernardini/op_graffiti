use crate::error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::fmt;

pub type Result<T, E = Report> = color_eyre::Result<T, E>;
// A generic error report
// Produced via `Err(some_err).wrap_err("Some context")`
// or `Err(color_eyre::eyre::Report::new(SomeError))`
pub struct Report(color_eyre::Report);

impl fmt::Debug for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Tell axum how to convert `Report` into a response.
impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let err = self.0;
        let err_string = format!("{err:?}");

        tracing::error!("{err_string}");

        if let Some(err) = err.downcast_ref::<error::Graffiti>() {
            return err.response();
        }

        // Fallback
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something panicked: {err_string}"),
        )
            .into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Graffiti {
    #[error("An error occurred: {0}")]
    Anyhow(#[from] anyhow::Error),
}
impl Graffiti {
    fn response(&self) -> Response {
        let (status, err_msg) = match self {
            Self::Anyhow(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("error getting wallet name {e}"),
            ),
        };
        (status, Json(json!({ "error": err_msg }))).into_response()
    }
}
