/// Represents a detected position opening event on Hyperliquid.
#[derive(Debug, Clone)]
pub struct PositionOpenEvent {
    pub coin: String,
    pub side: String, // "B" (Buy) or "S" (Sell)
    pub px: String,   // Price as string
    pub sz: String,   // Size as string
    pub time: u64,    // Epoch timestamp in milliseconds
    pub tid: u64,     // Unique trade/fill ID
}
