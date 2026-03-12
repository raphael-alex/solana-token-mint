use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintToChecked, TokenAccount, TokenInterface};

use crate::{config, Configuration, CustomError, MAX_ISSUANCE_PER_CALL};

pub fn handler_increase_issuance(ctx: Context<MintToken>, amount: u64) -> Result<()> {
    msg!("increase issuance by: {}", amount);

    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    require!(
        amount <= MAX_ISSUANCE_PER_CALL,
        CustomError::ExcessiveIssuance
    );

    let current_supply = ctx.accounts.mint.supply;
    require!(
        current_supply.saturating_add(amount) <= config::MAX_SUPPLY,
        CustomError::SupplyOverflow
    );

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[ctx.accounts.config.bump_vault_authority],
    ]];

    let mint_to_cpi_accounts = MintToChecked {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::mint_to_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_to_cpi_accounts,
        )
        .with_signer(signer_seeds),
        amount,
        ctx.accounts.mint.decimals,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"stm_config"],
        bump,
        has_one = admin @ CustomError::Unauthorized
    )]
    pub config: Account<'info, Configuration>,
    #[account(
        mut,
        seeds = [b"stm_mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"stm_vault"],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault_authority"],
        bump = config.bump_vault_authority
    )]
    pub vault_authority: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}
