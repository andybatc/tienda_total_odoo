use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use odoo_shop::{
    app::App,
    workers::webhook::{WebhookWorker, WebhookWorkerArgs},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_run_webhook_worker() {
    let boot = boot_test::<App>().await.unwrap();

    // Execute the worker ensuring that it operates in 'ForegroundBlocking' mode, which prevents the addition of your worker to the background
    assert!(
        WebhookWorker::perform_later(&boot.app_context,WebhookWorkerArgs {odoo_id: 13})
            .await
            .is_ok()
    );
    // Include additional assert validations after the execution of the worker
}
