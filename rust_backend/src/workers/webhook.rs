use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use sea_orm::{sea_query, Database, Set};
use crate::models::_entities::products;
pub struct WebhookWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WebhookWorkerArgs {
    pub odoo_id: i32,
}

#[async_trait]
impl BackgroundWorker<WebhookWorkerArgs> for WebhookWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    fn class_name() -> String {
        "Webhook".to_string()
    }

    async fn perform(&self, args: WebhookWorkerArgs) -> Result<()> {
        println!("================= Odoo Webhook Event =================");
        println!("🔄 Sincronizando producto específico. ID Odoo: {}", args.odoo_id);

        // 1. Conexión a la base de datos de Odoo 18
        let odoo_uri = "postgres://odoo:postgres@localhost:5432/odoo_prod";
        let odoo_db = Database::connect(odoo_uri).await
            .map_err(|e| Error::wrap(e))?;

        // 2. Buscar el producto específico en Odoo
        use crate::models::product_template_odoo;
        let odoo_item = product_template_odoo::Entity::find_by_id(args.odoo_id)
            .one(&odoo_db)
            .await?;

        if let Some(item) = odoo_item {
            // 1. Manejo seguro de JsonValue para el nombre
            let product_name = item.name.as_str().unwrap_or("Sin nombre");

            // 2. Mapear al modelo local (usando nombres correctos: name, sku, etc.)
            let active_item = products::ActiveModel {
                name: Set(Some(product_name.to_string())),
                // Si list_price es Option<Decimal>, lo pasamos directamente
                price: Set(item.list_price),
                odoo_id: Set(Some(item.id)), // Envolviendo en Some() para Option<i32>
                ..Default::default()
            };

            // 3. Upsert con nombres de columna corregidos
            products::Entity::insert(active_item)
                .on_conflict(
                    sea_query::OnConflict::column(products::Column::OdooId)
                        .update_columns([
                            products::Column::Name,
                            products::Column::Price
                        ])
                        .to_owned(),
                )
                .exec(&self.ctx.db)
                .await?;

            println!("✅ Producto {} (ID: {}) sincronizado.", product_name, item.id);
        } else {
            println!("⚠️ No se encontró el producto con ID {} en Odoo.", args.odoo_id);
        }

        Ok(())
    }
}
