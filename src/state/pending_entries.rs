use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::constants::*;

/// A single lottery entry recorded by the Transfer Hook on a buy.
///
/// Uses `#[zero_copy]` so it can live inside the zero-copy `PendingEntries` account
/// without blowing the SBF stack limit during deserialization.
#[zero_copy]
#[derive(Debug, Default)]
#[repr(C)]
pub struct LotteryEntry {
    /// The buyer's wallet pubkey (destination token account owner).
    pub buyer: Pubkey,
    /// Amount of tokens transferred (in mint's smallest denomination).
    pub amount: u64,
    /// Solana slot at which the buy occurred.
    pub slot: u64,
}

impl LotteryEntry {
    pub const LEN: usize = 32 + 8 + 8; // 48 bytes
}

/// Ring buffer of pending lottery entries for a creator mint.
///
/// Seeds: `[PENDING_ENTRIES_SEED, creator_mint.key()]`
///
/// Uses zero-copy deserialization (`AccountLoader`) to avoid placing the
/// 12KB buffer on the SBF stack. The runtime memory-maps the account data
/// directly, keeping stack usage minimal.
///
/// The keeper relays this buffer periodically and forwards entries to Base.
/// Overflow policy: drop-oldest (head advances, oldest overwritten).
#[account(zero_copy)]
#[derive(Debug)]
#[repr(C)]
pub struct PendingEntries {
    /// The creator mint this buffer belongs to.
    pub creator_mint: Pubkey,

    /// Write pointer — next slot to write into.
    pub head: u32,

    /// Number of entries currently in the buffer (0..=MAX_PENDING_ENTRIES).
    pub count: u32,

    /// Total number of entries dropped due to overflow.
    /// Keepr monitors this counter between polls.
    pub overflow_count: u64,

    /// Bump seed for PDA derivation.
    pub bump: u8,

    /// Alignment padding (zero-copy requires C-repr alignment).
    pub _padding: [u8; 7],

    /// The ring buffer itself.
    pub entries: [LotteryEntry; MAX_PENDING_ENTRIES],
}

impl PendingEntries {
    /// Account discriminator (8) + fields.
    /// 32 + 4 + 4 + 8 + 1 + 7(padding) + (48 * 256) = 32 + 4 + 4 + 8 + 1 + 7 + 12288 = 12344
    /// Total with discriminator: 8 + 12344 = 12352
    pub const LEN: usize =
        8 + 32 + 4 + 4 + 8 + 1 + 7 + (LotteryEntry::LEN * MAX_PENDING_ENTRIES);

    /// Push a new entry into the ring buffer.
    /// If the buffer is full, the oldest entry is overwritten (drop-oldest).
    pub fn push(&mut self, entry: LotteryEntry) -> bool {
        let idx = self.head as usize;
        let was_full = self.count as usize >= MAX_PENDING_ENTRIES;

        self.entries[idx] = entry;
        self.head = ((idx + 1) % MAX_PENDING_ENTRIES) as u32;

        if was_full {
            self.overflow_count += 1;
        } else {
            self.count += 1;
        }

        was_full
    }

    /// Returns true if the buffer has exceeded the emergency relay threshold.
    pub fn needs_emergency_relay(&self) -> bool {
        self.count as usize >= EMERGENCY_RELAY_THRESHOLD
    }
}
