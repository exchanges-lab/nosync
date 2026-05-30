use crate::structs::SharedMessage;
use thiserror::Error;
use tracing::{debug, info};

/// Errors specific to ModuleA.
#[derive(Error, Debug)]
pub enum ModuleAError {
    #[error("Failed to process message: {0}")]
    ProcessError(String),
}

/// ModuleA processes messages and manages a name field.
pub struct ModuleA {
    name: String,
}

impl ModuleA {
    /// Creates a new ModuleA instance.
    pub fn new(name: String) -> Result<Self, ModuleAError> {
        info!(name = %name, "Initializing ModuleA");
        Ok(Self { name })
    }

    /// Processes a shared message asynchronously.
    pub async fn process_message(&self, msg: SharedMessage) -> Result<String, ModuleAError> {
        debug!(msg_id = msg.id, "Processing message in ModuleA");
        if msg.content.is_empty() {
            return Err(ModuleAError::ProcessError(
                "Empty message content".to_string(),
            ));
        }
        Ok(format!("ModuleA[{}] processed: {}", self.name, msg.content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_message_success() {
        let module_a = ModuleA::new("test-a".to_string()).unwrap();
        let msg = SharedMessage {
            id: 42,
            content: "Hello".to_string(),
        };
        let result = module_a.process_message(msg).await.unwrap();
        assert_eq!(result, "ModuleA[test-a] processed: Hello");
    }

    #[tokio::test]
    async fn test_process_message_empty_content() {
        let module_a = ModuleA::new("test-a".to_string()).unwrap();
        let msg = SharedMessage {
            id: 42,
            content: String::new(),
        };
        let result = module_a.process_message(msg).await;
        assert!(result.is_err());
    }
}
