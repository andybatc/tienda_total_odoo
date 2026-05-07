use odoo_shop::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_get_products_webhooks() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/products_webhooks/").await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}

