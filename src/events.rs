use anchor_lang::prelude::*;

/// Emitted when a buy is detected and a lottery entry is recorded.
#[event]
pub struct LotteryEntryRecorded {
    pub creator_mint: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub slot: u64,
    pub buffer_count: u32,
}

/// Emitted when entries are relayed by the keeper for delivery to Base.
#[event]
pub struct EntriesRelayed {
    pub creator_mint: Pubkey,
    pub count: u32,
    pub overflow_count: u64,
}

/// Emitted when fees are settled by the keeper.
#[event]
pub struct FeesSettled {
    pub creator_mint: Pubkey,
    pub amount: u64,
}

/// Emitted when a winner is recorded on Solana.
#[event]
pub struct WinnerNotified {
    pub creator_mint: Pubkey,
    pub winner: Pubkey,
    pub shares_paid: u64,
    pub timestamp: i64,
}

/// Emitted when the keeper authority is rotated.
#[event]
pub struct KeeperRotated {
    pub creator_mint: Pubkey,
    pub old_keeper: Pubkey,
    pub new_keeper: Pubkey,
}

/// Emitted when entries are dropped due to buffer overflow.
#[event]
pub struct EntryOverflow {
    pub creator_mint: Pubkey,
    pub total_overflow_count: u64,
}
