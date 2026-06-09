use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::CreatorShareHookError;
use crate::events::*;
use crate::state::*;

/// Keeper-only: record a lottery winner on Solana.
///
/// Called by the Keepr `keepr-solana-winner-relay` workflow after a win
/// is detected on Base via the `LotteryWinnerNotification` event.
///
/// The frontend subscribes to the `WinnerNotified` event and the
/// WinnerRecord PDA to show "You won!" notifications.
#[derive(Accounts)]
pub struct RecordWinner<'info> {
    /// The keeper authority (must match `creator_config.keeper_authority`).
    pub keeper: Signer<'info>,

    /// CreatorConfig PDA — used to verify keeper authority.
    #[account(
        seeds = [CREATOR_CONFIG_SEED, creator_mint.key().as_ref()],
        bump = creator_config.bump,
        constraint = creator_config.keeper_authority == keeper.key() @ CreatorShareHookError::UnauthorizedKeeper,
    )]
    pub creator_config: Box<Account<'info, CreatorConfig>>,

    /// The Token-2022 mint (used for PDA derivation).
    /// CHECK: Only used as a seed — validated via PDA constraints.
    pub creator_mint: UncheckedAccount<'info>,

    /// WinnerRecord PDA — mutable to update with new winner.
    #[account(
        mut,
        seeds = [WINNER_RECORD_SEED, creator_mint.key().as_ref()],
        bump = winner_record.bump,
    )]
    pub winner_record: Box<Account<'info, WinnerRecord>>,
}

pub fn handler(ctx: Context<RecordWinner>, winner: Pubkey, shares_paid: u64) -> Result<()> {
    let clock = Clock::get()?;

    let record = &mut ctx.accounts.winner_record;
    record.winner = winner;
    record.shares_paid = shares_paid;
    record.timestamp = clock.unix_timestamp;

    emit!(WinnerNotified {
        creator_mint: ctx.accounts.creator_config.creator_mint,
        winner,
        shares_paid,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
