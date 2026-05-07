#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use crate::models::_entities::configs;
use sea_orm::Set;

#[debug_handler]
pub async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    format::empty()
}

#[derive(Serialize, Deserialize)]
pub struct TokenForm {
    pub token: String,
}
#[derive(Serialize, Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

async fn render_ui(State(ctx): State<AppContext>) -> Result<Response> {
    let config = configs::Entity::find()
        .filter(configs::Column::Key.eq("webhook_token"))
        .one(&ctx.db)
        .await?;

    let current_token = config
        .and_then(|c| c.value)
        .unwrap_or_else(|| "no_set".to_string());

    // Leemos el archivo manualmente y reemplazamos la variable
    let mut html = std::fs::read_to_string("assets/views/config/ui.html")
        .map_err(|_| Error::string("No se encuentra la plantilla HTML"))?;

    // Reemplazo simple (muy rudimentario, pero funciona sin Tera)
    html = html.replace("{{ current_token }}", &current_token);

    format::html(&html)
}

async fn handle_ui_update(
    State(ctx): State<AppContext>,
    // Usamos Form en lugar de Json para capturar el envío del navegador
    const_form: axum::extract::Form<TokenForm>,
) -> Result<Response> {
    let payload = const_form.0;

    let config = configs::Entity::find()
        .filter(configs::Column::Key.eq("webhook_token"))
        .one(&ctx.db)
        .await?;

    if let Some(c) = config {
        let mut active_model: configs::ActiveModel = c.into();
        active_model.value = Set(Some(payload.token));
        active_model.update(&ctx.db).await?;
    } else {
        configs::ActiveModel {
            key: Set(Some("webhook_token".to_string())),
            value: Set(Some(payload.token)),
            ..Default::default()
        }
            .insert(&ctx.db)
            .await?;
    }

    // Después de guardar, refrescamos la página
    format::render().redirect("/api/config/ui")
}

// --- LOGICA DE LA API (JSON) ---

async fn get_token(State(ctx): State<AppContext>) -> Result<Response> {
    let config = configs::Entity::find()
        .filter(configs::Column::Key.eq("webhook_token"))
        .one(&ctx.db)
        .await?;

    let token_value = config
        .and_then(|c| c.value)
        .unwrap_or_else(|| "not_set".to_string());

    format::json(token_value)
}

async fn update_token(
    State(ctx): State<AppContext>,
    Json(payload): Json<TokenRequest>,
) -> Result<Response> {
    let config = configs::Entity::find()
        .filter(configs::Column::Key.eq("webhook_token"))
        .one(&ctx.db)
        .await?;

    if let Some(c) = config {
        let mut active_model: configs::ActiveModel = c.into();
        active_model.value = Set(Some(payload.token));
        active_model.update(&ctx.db).await?;
    } else {
        configs::ActiveModel {
            key: Set(Some("webhook_token".to_string())),
            value: Set(Some(payload.token)),
            ..Default::default()
        }
            .insert(&ctx.db)
            .await?;
    }

    format::json(serde_json::json!({ "status": "ok" }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/config")
        // Endpoints para Odoo (JSON)
        .add("/token", get(get_token))
        .add("/token", post(update_token))
        // Endpoints para ti (Navegador/HTML)
        .add("/ui", get(render_ui))
        .add("/ui", post(handle_ui_update))
}