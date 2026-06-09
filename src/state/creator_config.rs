use anchor_lang::prelude::*;

use crate::constants::*;

/// Per-creator configuration PDA.
///
/// Seeds: `[CREATOR_CONFIG_SEED, creator_mint.key()]`
///
/// Stores fee parameters, keeper authority, known AMM programs for buy
/// detection, and lottery enablement flag.
#[account]
#[derive(Debug)]
pub struct CreatorConfig {
    /// The SPL Token-2022 mint this config belongs to.
    pub creator_mint: Pubkey,

    /// Authority that can update this config (typically deployer multisig).
    pub authority: Pubkey,

    /// Authorized keeper pubkey — the only signer allowed to call
    /// `settle_fees`, `relay_entries`, and `record_winner`.
    pub keeper_authority: Pubkey,

    /// Hub Creator Coin address (Base) encoded as bytes32.
    pub hub_creator_coin: [u8; 32],

    /// Hub ShareOFT address (Base) encoded as bytes32.
    pub hub_share_oft: [u8; 32],

    /// Fee in basis points (informational — actual fee is enforced by
    /// TransferFeeConfig on the mint, not by this program).
    pub fee_bps: u16,

    /// Minimum fee amount (in token smallest units) before fee settlement
    /// will execute. Set to 0 to settle on every call.
    pub settlement_threshold: u64,

    /// Whether lottery entry recording is enabled.
    /// When false, the hook still executes but does not write entries.
    pub lottery_enabled: bool,

    /// Number of known AMM programs currently stored.
    pub amm_program_count: u8,

    /// Known AMM program IDs for buy detection.
    /// If the source token account owner matches any of these, the
    /// transfer is classified as a "buy" and a lottery entry is recorded.
    pub known_amm_programs: [Pubkey; MAX_AMM_PROGRAMS],

    /// Bump seed for PDA derivation.
    pub bump: u8,

    /// Reserved space for future upgrades.
    pub _reserved: [u8; 64],
}

impl CreatorConfig {
    /// Account discriminator (8) + fields.
    /// Pubkey = 32, bytes32 = 32, u16 = 2, u64 = 8, bool = 1, u8 = 1, [Pubkey; 8] = 256, u8 = 1, [u8; 64] = 64
    /// Total: 8 + 32 + 32 + 32 + 32 + 32 + 2 + 8 + 1 + 1 + 256 + 1 + 64 = 501
    pub const LEN: usize =
        8 + 32 + 32 + 32 + 32 + 32 + 2 + 8 + 1 + 1 + (32 * MAX_AMM_PROGRAMS) + 1 + 64;

    /// Check if a pubkey is in the known AMM program list.
    pub fn is_known_amm(&self, program_id: &Pubkey) -> bool {
        let count = self.amm_program_count as usize;
        self.known_amm_programs[..count]
            .iter()
            .any(|p| p == program_id)
    }

    /// Add a program to the known AMM list. Returns error if full or duplicate.
    pub fn add_amm_program(&mut self, program_id: Pubkey) -> Result<()> {
        let count = self.amm_program_count as usize;
        if count >= MAX_AMM_PROGRAMS {
            return err!(crate::errors::CreatorShareHookError::AmmListFull);
        }
        if self.is_known_amm(&program_id) {
            return err!(crate::errors::CreatorShareHookError::AmmAlreadyExists);
        }
        self.known_amm_programs[count] = program_id;
        self.amm_program_count += 1;
        Ok(())
    }

    /// Remove a program from the known AMM list. Returns error if not found.
    pub fn remove_amm_program(&mut self, program_id: &Pubkey) -> Result<()> {
        let count = self.amm_program_count as usize;
        if let Some(idx) = self.known_amm_programs[..count]
            .iter()
            .position(|p| p == program_id)
        {
            // Swap-remove: move last element into the removed slot.
            self.known_amm_programs[idx] = self.known_amm_programs[count - 1];
            self.known_amm_programs[count - 1] = Pubkey::default();
            self.amm_program_count -= 1;
            Ok(())
        } else {
            err!(crate::errors::CreatorShareHookError::AmmNotFound)
        }
    }
}
