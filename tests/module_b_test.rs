use nosync::{ModuleA, ModuleB};

#[tokio::test]
async fn test_integration_module_b() {
    let module_a =
        ModuleA::new("integration-b-processor".to_string()).expect("Failed to create ModuleA");
    let module_b = ModuleB::new(module_a).expect("Failed to create ModuleB");
    let res = module_b.run().await;
    assert!(res.is_ok());
}
