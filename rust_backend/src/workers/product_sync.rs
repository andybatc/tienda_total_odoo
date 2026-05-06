use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use crate::models::_entities::product_template;
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

        // 1. Conexión a la base de datos de Odoo
        let odoo_uri = "postgres://odoo:postgres@localhost:5432/odoo_prod";
        let odoo_db = Database::connect(odoo_uri)
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        // 2. Traer productos publicados de Odoo
        let odoo_products = product_template::Entity::find()
            .filter(product_template::Column::IsPublished.eq(true))
            .all(&odoo_db)
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        println!("📦 Se encontraron {} productos para sincronizar.", odoo_products.len());

        for item in odoo_products {
            // --- TRANSFORMACIÓN ---
            // Odoo 18 usa JSON para el nombre. Extraemos el español o inglés.
            let name_string = item.name.get("es_ES")
                .or(item.name.get("en_US"))
                .and_then(|v| v.as_str())
                .unwrap_or("Sin nombre");

            // --- PREPARAR MODELO LOCAL ---
            let active_product = products::ActiveModel {
                odoo_id: Set(Some(item.id)),
                name: Set(Some(name_string.to_string())),
                sku: Set(item.default_code.clone()),
                price: Set(Some(item.list_price.unwrap_or_default())),
                stock: Set(Some(0.0)),
                ..Default::default()
            };

            // --- UPSERT (Insertar o Actualizar) ---
            products::Entity::insert(active_product)
                .on_conflict(
                    OnConflict::column(products::Column::OdooId)
                        .update_columns([
                            products::Column::Name,
                            products::Column::Sku,
                            products::Column::Price,
                        ])
                        .to_owned()
                )
                .exec(&self.ctx.db) // Se guarda en la DB de Loco
                .await
                .map_err(|e| Error::BadRequest(e.to_string()))?;
        }

        println!("✅ Sincronización finalizada exitosamente.");
        Ok(())
    }
}