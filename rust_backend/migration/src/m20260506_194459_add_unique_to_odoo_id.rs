use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx-products-odoo-id-unique")
                    .table(Products::Table)
                    .col(Products::OdooId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Usamos drop_index en lugar de drop
        manager
            .drop_index(
                Index::drop()
                    .name("idx-products-odoo-id-unique")
                    .table(Products::Table)
                    .to_owned()
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    OdooId,
}
