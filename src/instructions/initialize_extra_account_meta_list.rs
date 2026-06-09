use anchor_lang::prelude::*;
use anchor_lang::system_program;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::constants::*;

/// Initialize the extra-account-meta-list PDA required by the Transfer Hook
/// interface. This tells the SPL runtime which additional accounts to pass
/// into the `execute` (transfer_hook) instruction.
///
/// Must be called once per mint before any transfers will invoke the hook.
#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    /// Payer for the PDA account creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Token-2022 mint this hook is attached to.
    /// CHECK: Only used as a seed; validated by the Transfer Hook runtime.
    pub mint: UncheckedAccount<'info>,

    /// The extra-account-meta-list PDA.
    /// CHECK: Initialized below via `ExtraAccountMetaList::init`.
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_META_LIST_SEED, mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CreatorConfig PDA — must already exist (passed as extra account).
    /// CHECK: We derive and validate the PDA address.
    #[account(
        seeds = [CREATOR_CONFIG_SEED, mint.key().as_ref()],
        bump,
    )]
    pub creator_config: UncheckedAccount<'info>,

    /// PendingEntries PDA — must already exist (passed as extra account).
    /// CHECK: We derive and validate the PDA address.
    #[account(
        seeds = [PENDING_ENTRIES_SEED, mint.key().as_ref()],
        bump,
    )]
    pub pending_entries: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
    // Define extra accounts the hook needs beyond the standard Transfer Hook interface accounts.
    // The Transfer Hook runtime will automatically resolve these PDAs.
    let extra_metas = vec![
        // CreatorConfig PDA
        ExtraAccountMeta::new_with_seeds(
            &[
                spl_tlv_account_resolution::seeds::Seed::Literal {
                    bytes: CREATOR_CONFIG_SEED.to_vec(),
                },
                // mint key is at index 1 in the standard Transfer Hook account list
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 1 },
            ],
            false, // is_signer
            false, // is_writable (read-only for buy detection)
        )?,
        // PendingEntries PDA
        ExtraAccountMeta::new_with_seeds(
            &[
                spl_tlv_account_resolution::seeds::Seed::Literal {
                    bytes: PENDING_ENTRIES_SEED.to_vec(),
                },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 1 },
            ],
            false, // is_signer
            true,  // is_writable (we write entries on buy)
        )?,
    ];

    // Calculate the required space.
    let account_size = ExtraAccountMetaList::size_of(extra_metas.len())?;

    // Allocate the account via CPI.
    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[u8]] = &[
        EXTRA_ACCOUNT_META_LIST_SEED,
        mint_key.as_ref(),
        &[ctx.bumps.extra_account_meta_list],
    ];

    system_program::create_account(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            system_program::CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.extra_account_meta_list.to_account_info(),
            },
            &[signer_seeds],
        ),
        Rent::get()?.minimum_balance(account_size),
        account_size as u64,
        &crate::ID,
    )?;

    // Initialize the meta list.
    let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_metas)?;

    Ok(())
}
