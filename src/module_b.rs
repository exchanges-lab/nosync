use crate::module_a::ModuleA;
use crate::structs::SharedMessage;
use thiserror::Error;
use tracing::info;

/// Errors specific to ModuleB.
#[derive(Error, Debug)]
pub enum ModuleBError {
    #[error("ModuleA error occurred: {0}")]
    AError(#[from] crate::module_a::ModuleAError),
}

/// ModuleB holds a reference to ModuleA and interacts with it.
pub struct ModuleB {
    processor: ModuleA,
}

impl ModuleB {
    /// Creates a new ModuleB instance by taking ownership of a ModuleA processor.
    pub fn new(processor: ModuleA) -> Result<Self, ModuleBError> {
        info!("Initializing ModuleB");
        Ok(Self { processor })
    }

    /// Runs the business logic of ModuleB.
    pub async fn run(&self) -> Result<(), ModuleBError> {
        let msg = SharedMessage {
            id: 100,
            content: "Hello from ModuleB".to_string(),
        };
        let result = self.processor.process_message(msg).await?;
        info!(result = %result, "ModuleB execution succeeded");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_b_run() {
        let module_a = ModuleA::new("test-a".to_string()).unwrap();
        let module_b = ModuleB::new(module_a).unwrap();
        let result = module_b.run().await;
        assert!(result.is_ok());
    }
}
