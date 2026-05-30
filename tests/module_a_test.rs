use nosync::{ModuleA, SharedMessage};

#[tokio::test]
async fn test_integration_module_a() {
    let module_a = ModuleA::new("integration-a".to_string()).expect("Failed to create ModuleA");
    let msg = SharedMessage {
        id: 999,
        content: "Integration content".to_string(),
    };
    let res = module_a
        .process_message(msg)
        .await
        .expect("Failed to process");
    assert!(res.contains("integration-a"));
    assert!(res.contains("Integration content"));
}
