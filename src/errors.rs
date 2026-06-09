use anchor_lang::prelude::*;

#[error_code]
pub enum CreatorShareHookError {
    #[msg("Unauthorized: caller is not the keeper authority")]
    UnauthorizedKeeper,

    #[msg("Unauthorized: caller is not the config authority")]
    UnauthorizedAuthority,

    #[msg("AMM program list is full (max 8)")]
    AmmListFull,

    #[msg("AMM program not found in allowlist")]
    AmmNotFound,

    #[msg("AMM program already in allowlist")]
    AmmAlreadyExists,

    #[msg("Lottery is not enabled for this creator")]
    LotteryDisabled,

    #[msg("No pending entries to relay")]
    NoEntriesToRelay,

    #[msg("Invalid mint — does not match config")]
    InvalidMint,

    #[msg("PendingEntries buffer overflow counter mismatch")]
    OverflowCounterMismatch,

    #[msg("Invalid fee BPS — must be <= 10000")]
    InvalidFeeBps,

    #[msg("Extra account meta list already initialized")]
    MetaListAlreadyInitialized,
}
