use std::fmt::Display;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u64)]
pub enum MarketStatus {
    Uninitialized,
    /// All new orders, placements, and reductions are accepted. Crossing the spread is permissionless.
    Active,
    /// Only places, reductions and withdrawals are accepted.
    PostOnly,
    /// Only reductions and withdrawals are accepted.
    Paused,
    /// Only reductions and withdrawals are accepted. The market authority can forcibly cancel
    /// all orders.
    Closed,
    /// Used to signal the market to be deleted. Can only be called in a Closed state where all orders
    /// and traders are removed from the book
    Tombstoned,
}

impl Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Uninitialized => write!(f, "Uninitialized"),
            MarketStatus::Active => write!(f, "Active"),
            MarketStatus::PostOnly => write!(f, "PostOnly"),
            MarketStatus::Paused => write!(f, "Paused"),
            MarketStatus::Closed => write!(f, "Closed"),
            MarketStatus::Tombstoned => write!(f, "Tombstoned"),
        }
    }
}

impl Default for MarketStatus {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl From<u64> for MarketStatus {
    fn from(status: u64) -> Self {
        match status {
            0 => Self::Uninitialized,
            1 => Self::Active,
            2 => Self::PostOnly,
            3 => Self::Paused,
            4 => Self::Closed,
            5 => Self::Tombstoned,
            _ => panic!("Invalid market status"),
        }
    }
}
