#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260506_184955_products;
mod m20260506_194459_add_unique_to_odoo_id;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260506_184955_products::Migration),
            Box::new(m20260506_194459_add_unique_to_odoo_id::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}