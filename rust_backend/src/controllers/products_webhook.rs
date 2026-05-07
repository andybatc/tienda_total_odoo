#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::workers::webhook::{WebhookWorker, WebhookWorkerArgs};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use std::time::Duration;
use axum::{
    error_handling::HandleErrorLayer,
    routing::post,
    Json,
    http::StatusCode,
};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
#[debug_handler]
pub async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    format::empty()
}

#[derive(Serialize, Deserialize)]
pub struct OdooPayload {
    pub odoo_id: i32,
}

async fn handle_rate_limit_error(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        format!("Límite de peticiones excedido: {}", err),
    )
}

async fn update(
    State(ctx): State<AppContext>,
    Json(args): Json<WebhookWorkerArgs>,
) -> Result<Response> {
    // Loco maneja todo el encolado (queue, serialización, etc.) con perform_later
    WebhookWorker::perform_later(&ctx, args).await?;

    format::json::<()>(())
}

pub fn routes() -> Routes {
    // Al añadir BufferLayer, hacemos que todo el middleware sea "Clone"
    // y Axum por fin lo aceptará felizmente.
    let middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_rate_limit_error))
        .layer(BufferLayer::new(1024)) 
        .layer(RateLimitLayer::new(10, Duration::from_secs(1)));

    Routes::new()
        .prefix("api/webhooks/odoo")
        .add("/update", post(update))
        .layer(middleware)
}