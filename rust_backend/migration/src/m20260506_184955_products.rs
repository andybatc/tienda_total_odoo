use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "products",
            &[
            
            ("id", ColType::PkAuto),
            
            ("odoo_id", ColType::IntegerNull),
            ("sku", ColType::StringNull),
            ("name", ColType::StringNull),
            ("price", ColType::DecimalNull),
            ("stock", ColType::FloatNull),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "products").await
    }
}
