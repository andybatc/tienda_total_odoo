use loco_rs::cli;
use migration::Migrator;
use odoo_shop::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}
