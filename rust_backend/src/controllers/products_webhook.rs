#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::workers::webhook::{WebhookWorker, WebhookWorkerArgs};

#[debug_handler]
pub async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    format::empty()
}

#[derive(Serialize, Deserialize)]
pub struct OdooPayload {
    pub odoo_id: i32,
}

pub async fn update(
    State(ctx): State<AppContext>,
    Json(payload): Json<OdooPayload>,
) -> Result<Response> {
    // ESTA ES LA CLAVE:
    // El controlador no hace el trabajo, solo lo "encola" para el Worker.
    WebhookWorker::perform_later(&ctx, WebhookWorkerArgs {
        odoo_id: payload.odoo_id
    }).await?;

    format::json(serde_json::json!({ "status": "enqueued", "id": payload.odoo_id }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/webhooks/odoo")
        .add("/update", post(update))
}
