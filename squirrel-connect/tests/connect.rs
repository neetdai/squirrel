use squirrel_connect::{
    Options,
    Manager,
};
use tokio;

#[tokio::test]
async fn test_connect() {
    let options = Options::from_url("mysql://root:123456@localhost:3306/test").unwrap();
    let mut manager = Manager::new();
    manager.add_master(&options).unwrap();
}