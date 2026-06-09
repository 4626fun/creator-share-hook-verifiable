use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU");

#[program]
pub mod creator_share_hook {
    use super::*;

    /// Initialize a new creator mint configuration.
    /// Creates CreatorConfig + PendingEntries PDAs for the given mint.
    pub fn initialize_creator(
        ctx: Context<InitializeCreator>,
        params: InitializeCreatorParams,
    ) -> Result<()> {
        instructions::initialize_creator::handler(ctx, params)
    }

    /// Initialize the extra account meta list required by the Transfer Hook interface.
    /// Must be called once per mint before the hook can fire.
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        instructions::initialize_extra_account_meta_list::handler(ctx)
    }

    /// Transfer Hook execute — named with Anchor discriminator.
    /// Also reachable via the SPL fallback below.
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        instructions::execute_hook::handler(ctx, amount)
    }

    /// Keeper-only: settle withheld fees via Token-2022 CPI.
    /// Fees are collected to a designated account for bridging to Base.
    pub fn settle_fees<'info>(ctx: Context<'info, SettleFees<'info>>) -> Result<()> {
        instructions::settle_fees::handler(ctx)
    }

    /// Keeper-only: emit and clear PendingEntries for relay to Base.
    /// Returns entries to the keeper for batch relay via SolanaBridgeAdapter.
    pub fn relay_entries(ctx: Context<RelayEntries>) -> Result<()> {
        instructions::relay_entries::handler(ctx)
    }

    /// Keeper-only: record a lottery winner on Solana.
    /// Called by Keepr after a win is detected on Base.
    pub fn record_winner(
        ctx: Context<RecordWinner>,
        winner: Pubkey,
        shares_paid: u64,
    ) -> Result<()> {
        instructions::record_winner::handler(ctx, winner, shares_paid)
    }

    // ── Admin ────────────────────────────────────────────────────────

    /// Update CreatorConfig parameters (fee_bps, fee settlement threshold, etc.).
    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        instructions::admin::update_config_handler(ctx, params)
    }

    /// Add a known AMM program to the allowlist.
    pub fn add_amm_program(ctx: Context<UpdateConfig>, program_id: Pubkey) -> Result<()> {
        instructions::admin::add_amm_program_handler(ctx, program_id)
    }

    /// Remove a known AMM program from the allowlist.
    pub fn remove_amm_program(ctx: Context<UpdateConfig>, program_id: Pubkey) -> Result<()> {
        instructions::admin::remove_amm_program_handler(ctx, &program_id)
    }

    /// Rotate the keeper authority to a new pubkey.
    pub fn rotate_keeper(ctx: Context<UpdateConfig>, new_keeper: Pubkey) -> Result<()> {
        instructions::admin::rotate_keeper_handler(ctx, new_keeper)
    }

    /// Fallback handler for the SPL Transfer Hook Interface.
    ///
    /// The Token-2022 runtime invokes the hook program using the Execute
    /// discriminator from `spl_transfer_hook_interface`, not an Anchor
    /// instruction discriminator. This fallback intercepts that call,
    /// deserializes the accounts manually, and delegates to the hook handler.
    pub fn fallback<'info>(
        program_id: &'info Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        // SPL Transfer Hook Execute discriminator:
        // first 8 bytes of `hashv(&[b"spl-transfer-hook-interface:execute"])`
        // = [105, 37, 101, 197, 75, 251, 102, 26]
        const EXECUTE_IX_TAG_LE: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

        if data.len() < 16 {
            return Err(ProgramError::InvalidInstructionData.into());
        }

        if data[..8] != EXECUTE_IX_TAG_LE {
            return Err(ProgramError::InvalidInstructionData.into());
        }

        // Remaining data after the 8-byte discriminator is `amount: u64` (LE)
        let amount = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        // Deserialize accounts using Anchor's try_accounts and run the handler
        // inside a scope so mutable borrows end before account exit.
        {
            let mut bumps = instructions::execute_hook::TransferHookBumps::default();
            let mut remaining = accounts;
            let mut reallocs = std::collections::BTreeSet::new();
            let mut ctx_accounts = instructions::execute_hook::TransferHook::try_accounts(
                program_id,
                &mut remaining,
                data,
                &mut bumps,
                &mut reallocs,
            )?;

            let _ = remaining;
            let _ = bumps;
            instructions::execute_hook::process_transfer_hook(&mut ctx_accounts, amount)?;

            // Serialize modified accounts back to their data buffers.
            // This is critical — without this call, writes to PendingEntries
            // made inside the handler would not be persisted.
            ctx_accounts.exit(program_id)?;
        }

        Ok(())
    }
}
