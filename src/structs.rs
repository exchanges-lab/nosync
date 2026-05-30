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
    pub oid: u64,     // Hyperliquid Order ID
    pub action: TradeAction,
    pub start_pos: String, // Position size before the trade
    pub end_pos: String,   // Position size after the trade
    pub crossed: bool,     // True if taker (market), false if maker (limit)
}

/// Represents the structured row data formatted for Notion database insertion.
#[derive(Debug, Clone)]
pub struct NotionRowData {
    pub symbol: String,       // e.g., "BTCUSDC"
    pub quantity: String,     // e.g., "0.02484"
    pub filled_price: String, // e.g., "72543"
    pub direction: String,    // "Long" or "Short"
    pub exchange: String,     // e.g., "Hyperliquid"
    pub date_time: String,    // Format: "YYYY/MM/DD HH:MM"
    pub time: u64,            // Unix timestamp in milliseconds
    pub order_id: u64,        // Hyperliquid order ID (oid)
    pub check: bool,          // false
    pub level: String,        // e.g. "0", "1", "2"
    pub order_type: String,   // "MARKET" or "LIMIT"
}

impl NotionRowData {
    /// Calculate Level from USD value:
    /// level 0 = 2.5$, level 1 = 5$, level 2 = 10$, and so on (scaling logarithmically by factor of 2)
    pub fn calculate_level(usd_val: f64) -> String {
        if usd_val <= 0.0 {
            "0".to_string()
        } else {
            let ratio = usd_val / 2.5;
            let lvl = ratio.log2().round() as i32;
            let lvl = lvl.max(0);
            lvl.to_string()
        }
    }

    /// Determine Order Type from the crossed flag:
    /// crossed == true represents MARKET (taker), crossed == false represents LIMIT (maker)
    pub fn determine_order_type(crossed: bool) -> String {
        if crossed {
            "MARKET".to_string()
        } else {
            "LIMIT".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_level() {
        // Test base cases specified by user
        assert_eq!(NotionRowData::calculate_level(2.5), "0");
        assert_eq!(NotionRowData::calculate_level(5.0), "1");
        assert_eq!(NotionRowData::calculate_level(10.0), "2");

        // Test boundary limits (midpoints between levels)
        // Midpoint of 2.5 and 5.0 is geometric mean sqrt(2.5 * 5.0) = ~3.535
        // log2(3.5 / 2.5) = log2(1.4) = ~0.485 (rounds to 0) -> level 0
        // log2(3.6 / 2.5) = log2(1.44) = ~0.526 (rounds to 1) -> level 1
        assert_eq!(NotionRowData::calculate_level(3.5), "0");
        assert_eq!(NotionRowData::calculate_level(3.6), "1");

        // Midpoint of 5.0 and 10.0 is geometric mean sqrt(5 * 10) = ~7.07
        // log2(7.0 / 2.5) = log2(2.8) = ~1.485 (rounds to 1) -> level 1
        // log2(7.1 / 2.5) = log2(2.84) = ~1.506 (rounds to 2) -> level 2
        assert_eq!(NotionRowData::calculate_level(7.0), "1");
        assert_eq!(NotionRowData::calculate_level(7.1), "2");

        // Test edge cases
        assert_eq!(NotionRowData::calculate_level(0.0), "0");
        assert_eq!(NotionRowData::calculate_level(-5.5), "0");
        assert_eq!(NotionRowData::calculate_level(1.0), "0");

        // Test higher levels
        assert_eq!(NotionRowData::calculate_level(20.0), "3");
        assert_eq!(NotionRowData::calculate_level(40.0), "4");
    }

    #[test]
    fn test_determine_order_type() {
        assert_eq!(NotionRowData::determine_order_type(true), "MARKET");
        assert_eq!(NotionRowData::determine_order_type(false), "LIMIT");
    }
}

