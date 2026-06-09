use anchor_lang::prelude::*;
use anchor_spl::token_2022;

use crate::constants::*;
use crate::errors::CreatorShareHookError;
use crate::events::*;
use crate::state::*;

/// Keeper-only: emit and clear all pending lottery entries for relay.
///
/// The keeper calls this to relay the PendingEntries ring buffer, then
/// relays the entries to Base via `SolanaBridgeAdapter.processLotteryEntryFromSolana()`.
///
/// Entries are emitted as an Anchor event for the keeper to read from
/// the transaction logs. The buffer is reset after relay.
#[derive(Accounts)]
pub struct RelayEntries<'info> {
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
    #[account(owner = token_2022::ID)]
    pub creator_mint: UncheckedAccount<'info>,

    /// PendingEntries PDA — zero-copy, mutable to emit + clear entries.
    #[account(
        mut,
        seeds = [PENDING_ENTRIES_SEED, creator_mint.key().as_ref()],
        bump,
        constraint = pending_entries.load()?.creator_mint == creator_mint.key() @ CreatorShareHookError::InvalidMint,
    )]
    pub pending_entries: AccountLoader<'info, PendingEntries>,
}

pub fn handler(ctx: Context<RelayEntries>) -> Result<()> {
    let creator_mint = ctx.accounts.creator_config.creator_mint;
    let mut pending = ctx.accounts.pending_entries.load_mut()?;

    if pending.count == 0 {
        return err!(CreatorShareHookError::NoEntriesToRelay);
    }

    let count = pending.count;
    let head = pending.head as usize;
    let max = MAX_PENDING_ENTRIES;

    let start = if (count as usize) < max { 0 } else { head };

    for i in 0..(count as usize) {
        let idx = (start + i) % max;
        let entry = &pending.entries[idx];
        emit!(LotteryEntryRecorded {
            creator_mint,
            buyer: entry.buyer,
            amount: entry.amount,
            slot: entry.slot,
            buffer_count: 0,
        });
    }

    let overflow_count = pending.overflow_count;

    pending.head = 0;
    pending.count = 0;

    emit!(EntriesRelayed {
        creator_mint,
        count,
        overflow_count,
    });

    Ok(())
}
