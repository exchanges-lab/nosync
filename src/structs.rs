/// Shared message structure across modules.
#[derive(Debug, Clone)]
pub struct SharedMessage {
    pub id: u64,
    pub content: String,
}
