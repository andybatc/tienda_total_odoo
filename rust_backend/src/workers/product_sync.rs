use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use crate::models::product_template_odoo;
use crate::models::_entities::products;
use sea_orm::{Database, sea_query::OnConflict};

pub struct Worker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkerArgs {
}

#[async_trait]
impl BackgroundWorker<WorkerArgs> for Worker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    fn class_name() -> String {
        "ProductSync".to_string()
    }

    async fn perform(&self, _args: WorkerArgs) -> Result<()> {
        println!("🚀 Iniciando Sincronización: Odoo 18 -> Base Local");

        let odoo_uri = "postgres://odoo:postgres@localhost:5432/odoo_prod";
        let odoo_db = Database::connect(odoo_uri)
            .await
            .map_err(|e| {
                println!("❌ Fallo conectando a Odoo: {}", e);
                Error::BadRequest(e.to_string())
            })?;

        let odoo_products = product_template_odoo::Entity::find()
            .filter(product_template_odoo::Column::IsPublished.eq(true))
            .all(&odoo_db)
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        println!("📦 Se encontraron {} productos en Odoo.", odoo_products.len());

        for item in odoo_products {
            let name_string = item.name.get("es_ES")
                .or(item.name.get("en_US"))
                .and_then(|v| v.as_str())
                .unwrap_or("Sin nombre");

            println!("🔄 Procesando: {} (ID Odoo: {})", name_string, item.id);

            let active_product = products::ActiveModel {
                odoo_id: Set(Some(item.id)),
                name: Set(Some(name_string.to_string())),
                sku: Set(item.default_code.clone()),
                price: Set(Some(item.list_price.unwrap_or_default())),
                stock: Set(Some(0.0)),
                ..Default::default()
            };

            match products::Entity::insert(active_product)
                .on_conflict(
                    OnConflict::column(products::Column::OdooId)
                        .update_columns([
                            products::Column::Name,
                            products::Column::Sku,
                            products::Column::Price,
                        ])
                        .to_owned()
                )
                .exec(&self.ctx.db)
                .await
            {
                Ok(res) => println!("   ✅ Guardado exitoso (ID Local: {:?})", res.last_insert_id),
                Err(err) => println!("   ❌ ERROR guardando {}: {}", name_string, err),
            }
        }

        println!("✅ Proceso terminado.");
        Ok(())
    }
}