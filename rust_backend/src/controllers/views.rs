#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use crate::controllers::config::TokenForm;
use crate::models::_entities::{configs, users};
use crate::models::users::LoginParams;

async fn login_display(State(_ctx): State<AppContext>) -> Result<Response> {
    // Esto te dirá en la terminal desde dónde se está ejecutando el programa
    if let Ok(current_dir) = std::env::current_dir() {
        println!("Directorio actual de ejecución: {:?}", current_dir);
    }

    let html_path = "assets/views/auth/login.html";
    let html = std::fs::read_to_string(html_path).map_err(|e| {
        // Imprime el error real de sistema (ej. Permission Denied o No such file)
        println!("Error leyendo el HTML ({}) : {:?}", html_path, e);
        Error::string("No se encuentra la plantilla de login")
    })?;

    format::html(&html)
}

async fn login_web(
    State(ctx): State<AppContext>,
    Form(params): Form<LoginParams>,
) -> Result<Response> {
    // 1. Buscar usuario (Corregido el manejo de errores)
    let user = users::Model::find_by_email(&ctx.db, &params.email)
        .await
        .map_err(|_| Error::string("Credenciales inválidas"))?;

    // 2. Verificar password
    if !user.verify_password(&params.password) {
        return Err(Error::string("Credenciales inválidas"));
    }

    // 3. Generar Token
    let jwt_config = ctx.config.get_jwt_config()?;
    let token = user
        .generate_jwt(&jwt_config.secret, jwt_config.expiration)
        .map_err(|e| Error::string(&e.to_string()))?;

    // 4. Crear Cookie
    let cookie = format!(
        "token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token, jwt_config.expiration
    );

    // 5. Respuesta (Usamos format::render().redirect o Response manual)
    // Para HTMX y redirección, esto es lo más limpio:
    Response::builder()
        .header("Set-Cookie", cookie)
        .header("HX-Redirect", "/ui/auth/token")
        .body(axum::body::Body::empty())
        .map_err(|e| Error::string(&e.to_string()))
}

async fn register_display() -> Result<Response> {
    let html = std::fs::read_to_string("assets/views/auth/register.html")
        .map_err(|_| Error::string("No se encuentra la plantilla de registro"))?;
    format::html(&html)
}

async fn render_ui(State(ctx): State<AppContext>) -> Result<Response> {
    // 1. Obtener el token (aquí pondrías tu lógica para sacar el token de la DB o Config)
    let config = configs::Entity::find()
        .filter(configs::Column::Key.eq("webhook_token"))
        .one(&ctx.db)
        .await
        .map_err(|e| {
            tracing::error!("Error consultando la DB: {:?}", e);
            Error::string("Error al conectar con la base de datos")
        })?;

    // 2. Extraer el valor o poner un mensaje si no existe
    let current_token = config
        .and_then(|c| c.value)
        .unwrap_or_else(|| "No configurado".to_string());

    // 2. Leer el archivo
    let html_content = std::fs::read_to_string("assets/views/config/ui.html")
        .map_err(|e| {
            tracing::error!("Error leyendo ui.html: {:?}", e);
            Error::string("Error al cargar la vista de configuración")
        })?;

    // 3. Inyectar el valor dinámico
    let rendered_html = html_content.replace("{CURRENT_TOKEN}", &current_token);

    format::html(&rendered_html)
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

    Response::builder()
        .header("HX-Refresh", "true")
        .body(axum::body::Body::empty())
        .map_err(|e| Error::string(&e.to_string()))
    // Después de guardar, refrescamos la página
    //format::render().redirect("ui/auth/token")
}

pub fn routes() -> Routes {
    Routes::new()
        // Grupo para Autenticación Web
        // URL Resultante: /api/ui/auth/web-login
        .prefix("ui/auth")
        .add("/web-login", get(login_display))
        .add("/web-login", post(login_web))
        .add("/web-register", get(register_display))
        // Grupo para Configuración
        // URL Resultante: /api/views/config/token
        .add("/token", get(render_ui))
        .add("/token", post(handle_ui_update))
}
