use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_2022_extensions::transfer_fee::{
    harvest_withheld_tokens_to_mint as cpi_harvest_withheld_tokens_to_mint,
    withdraw_withheld_tokens_from_mint as cpi_withdraw_withheld_tokens_from_mint,
    HarvestWithheldTokensToMint,
    WithdrawWithheldTokensFromMint,
};
use anchor_spl::token_interface::TokenAccount as TokenInterfaceAccount;
use anchor_spl::token_interface::Mint as MintInterface;
use anchor_spl::token_interface;

use crate::constants::*;
use crate::errors::CreatorShareHookError;
use crate::events::*;
use crate::state::*;

/// Keeper-only: settle withheld TransferFeeConfig fees from the mint
/// into a designated fee vault token account.
///
/// After this instruction, the keeper bridges the collected fees to Base
/// via `SolanaBridgeAdapter.receiveFeeFromSolana()`.
#[derive(Accounts)]
pub struct SettleFees<'info> {
    /// The keeper authority (must match `creator_config.keeper_authority`).
    pub keeper: Signer<'info>,

    /// CreatorConfig PDA — used to verify keeper authority.
    #[account(
        seeds = [CREATOR_CONFIG_SEED, mint.key().as_ref()],
        bump = creator_config.bump,
        constraint = creator_config.keeper_authority == keeper.key() @ CreatorShareHookError::UnauthorizedKeeper,
    )]
    pub creator_config: Box<Account<'info, CreatorConfig>>,

    /// The Token-2022 mint with TransferFeeConfig extension.
    #[account(mut)]
    pub mint: InterfaceAccount<'info, MintInterface>,

    /// The destination token account to receive harvested fees.
    /// Typically owned by the keeper or a fee collection wallet.
    /// CHECK: Must be a valid Token-2022 token account for this mint.
    #[account(mut)]
    pub fee_vault: UncheckedAccount<'info>,

    /// Token-2022 program.
    pub token_program: Program<'info, Token2022>,
}

pub fn handler<'info>(ctx: Context<'info, SettleFees<'info>>) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();

    // Validate fee_vault is a Token-2022 account for this mint.
    let fee_vault_info = ctx.accounts.fee_vault.to_account_info();
    let fee_vault_data = fee_vault_info.try_borrow_data()?;
    let mut fee_vault_data_slice: &[u8] = &fee_vault_data;
    let fee_vault_state = TokenInterfaceAccount::try_deserialize(&mut fee_vault_data_slice)?;
    if fee_vault_state.mint != mint_key {
        return err!(CreatorShareHookError::InvalidMint);
    }

    let amount_before = fee_vault_state.amount;
    drop(fee_vault_data);

    // Step 1: Harvest withheld fees from token accounts to the mint.
    // The token accounts are provided as remaining_accounts.
    if !ctx.remaining_accounts.is_empty() {
        let sources: Vec<AccountInfo<'_>> = ctx
            .remaining_accounts
            .iter()
            .map(|a| a.to_account_info())
            .collect();
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.key(),
            HarvestWithheldTokensToMint {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        );
        cpi_harvest_withheld_tokens_to_mint(cpi_ctx, sources)?;
    }

    // Step 2: Withdraw all withheld tokens from the mint into fee_vault.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.key(),
        WithdrawWithheldTokensFromMint {
            token_program_id: ctx.accounts.token_program.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            destination: fee_vault_info.clone(),
            authority: ctx.accounts.keeper.to_account_info(),
        },
    );
    cpi_withdraw_withheld_tokens_from_mint(cpi_ctx)?;

    // Re-read fee_vault to compute actual withdrawn amount.
    let fee_vault_data_after = fee_vault_info.try_borrow_data()?;
    let mut fee_vault_data_after_slice: &[u8] = &fee_vault_data_after;
    let fee_vault_state_after =
        token_interface::TokenAccount::try_deserialize(&mut fee_vault_data_after_slice)?;
    let amount_after = fee_vault_state_after.amount;
    let delta = amount_after.saturating_sub(amount_before);

    emit!(FeesSettled {
        creator_mint: ctx.accounts.creator_config.creator_mint,
        amount: delta,
    });

    Ok(())
}
