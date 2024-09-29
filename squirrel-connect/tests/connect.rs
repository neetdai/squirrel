use squirrel_connect::{Manager, Options};
use tokio;

#[tokio::test]
async fn test_connect() {
    let options = Options::from_url("mysql://test:123456@172.20.58.113:3306/test").unwrap();
    let mut manager = Manager::new();
    manager.add_connect_options(&options).unwrap();

    manager.run().await.unwrap();
}
