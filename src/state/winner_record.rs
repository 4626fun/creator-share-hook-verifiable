use anchor_lang::prelude::*;

/// Latest lottery winner record for a creator mint.
///
/// Seeds: `[WINNER_RECORD_SEED, creator_mint.key()]`
///
/// Written by the keeper when a win is detected on Base.
/// Frontend subscribes to this PDA for "You won!" notifications.
#[account]
#[derive(Debug)]
pub struct WinnerRecord {
    /// The creator mint this record belongs to.
    pub creator_mint: Pubkey,

    /// The winner's Solana wallet pubkey.
    pub winner: Pubkey,

    /// Amount of vault shares paid to the winner (in Base token units).
    pub shares_paid: u64,

    /// Unix timestamp when the win was recorded on Solana.
    pub timestamp: i64,

    /// Bump seed for PDA derivation.
    pub bump: u8,
}

impl WinnerRecord {
    /// Account discriminator (8) + 32 + 32 + 8 + 8 + 1 = 89
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1;
}
