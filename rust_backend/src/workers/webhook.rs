use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use sea_orm::{Database, Set};
use crate::models::_entities::products;
use std::time::Duration;
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
        tokio::time::sleep(Duration::from_millis(500)).await;
        let odoo_uri = "postgres://odoo:postgres@localhost:5432/odoo_prod";
        let odoo_db = Database::connect(odoo_uri).await
            .map_err(|e| Error::wrap(e))?;

        // 2. Traer datos frescos de Odoo
        use crate::models::product_template_odoo;
        let odoo_item = product_template_odoo::Entity::find_by_id(args.odoo_id)
            .one(&odoo_db)
            .await
            .map_err(|e| Error::wrap(e))?;

        if let Some(item) = odoo_item {
            // LÓGICA ODOO 18: El nombre es un JSONB {"es_ES": "Nombre", "en_US": "Name"}
            // Intentamos obtener español, luego inglés, luego cualquier valor, o "Sin nombre"
            let product_name = item.name
                .get("es_ES")
                .or_else(|| item.name.get("en_US"))
                .or_else(|| item.name.as_object().and_then(|obj| obj.values().next()))
                .and_then(|v| v.as_str())
                .unwrap_or("Sin nombre");

            // 4. Buscar en la base de datos local de la tienda (Rust)
            let local_product = products::Entity::find()
                .filter(products::Column::OdooId.eq(item.id))
                .one(&self.ctx.db)
                .await?;

            match local_product {
                Some(existing_product) => {
                    println!("🔄 Actualizando producto: {} (Odoo ID: {})", product_name, item.id);

                    let mut active_model: products::ActiveModel = existing_product.into();

                    // Solo actualizamos si el nombre extraído es válido
                    if product_name != "Sin nombre" {
                        active_model.name = Set(Some(product_name.to_string()));
                    }

                    active_model.price = Set(item.list_price);
                    active_model.update(&self.ctx.db).await?;
                }
                None => {
                    println!("✨ Creando nuevo producto: {} (Odoo ID: {})", product_name, item.id);

                    let new_product = products::ActiveModel {
                        name: Set(Some(product_name.to_string())),
                        price: Set(item.list_price),
                        odoo_id: Set(Some(item.id)),
                        ..Default::default()
                    };

                    new_product.insert(&self.ctx.db).await?;
                }
            }

            println!("✅ Sincronización exitosa para ID: {}", item.id);
        } else {
            println!("⚠️ No se encontró el producto con ID {} en la base de datos de Odoo", args.odoo_id);
        }
        Ok(())
    }
}
