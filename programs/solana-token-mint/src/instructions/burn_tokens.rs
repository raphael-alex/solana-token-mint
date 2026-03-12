use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, BurnChecked, Mint, TokenAccount, TokenInterface},
};

use crate::{Configuration, CustomError};

pub fn handler_burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
    msg!("Burn {} tokens ", amount,);
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    require!(amount > 0, CustomError::AmountCannotBeZero);
    require!(
        amount <= ctx.accounts.vault.amount,
        CustomError::InsufficientAirdropAmount,
    );
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[ctx.accounts.config.bump_vault_authority],
    ]];
    let burn_cpi_accounts = BurnChecked {
        from: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            burn_cpi_accounts,
        )
        .with_signer(signer_seeds),
        amount,
        ctx.accounts.mint.decimals,
    )?;
    Ok(())
}

// burn tokens
#[derive(Accounts)]
pub struct BurnTokens<'info> {
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
        seeds = [b"vault_authority"],
        bump = config.bump_vault_authority
    )]
    pub vault_authority: SystemAccount<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"stm_vault"],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
