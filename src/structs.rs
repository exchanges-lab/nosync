/// The action type of a trade fill relative to the position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeAction {
    /// Opening a position (initial position size was 0)
    Open,
    /// Adding to an existing position (absolute position size increases)
    Increase,
    /// Reducing an existing position (absolute position size decreases but remains non-zero)
    Decrease,
    /// Fully closing an existing position (resulting position size becomes 0)
    Close,
}

/// Represents a detected trade event on Hyperliquid affecting a wallet's position.
#[derive(Debug, Clone)]
pub struct PositionTradeEvent {
    pub coin: String,
    pub side: String, // "B" (Buy) or "S" (Sell)
    pub px: String,   // Price as string
    pub sz: String,   // Trade size as string
    pub time: u64,    // Epoch timestamp in milliseconds
    pub tid: u64,     // Unique trade ID
    pub action: TradeAction,
    pub start_pos: String, // Position size before the trade
    pub end_pos: String,   // Position size after the trade
}
