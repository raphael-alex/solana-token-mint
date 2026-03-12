use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintToChecked, TokenAccount, TokenInterface},
};

use crate::{Configuration, CustomError, MAX_INITIAL_SUPPLY};

pub fn handler_mint(ctx: Context<CreateAndMintTokens>, amount: u64) -> Result<()> {
    let seeds = &[b"vault_authority".as_ref(), &[ctx.bumps.vault_authority]];

    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    require!(
        amount <= MAX_INITIAL_SUPPLY,
        CustomError::ExcessiveInitialSupply
    );

    let mint_cpi_accounts = MintToChecked {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::mint_to_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_cpi_accounts,
        )
        .with_signer(&[seeds]),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    let config = &mut ctx.accounts.config;
    config.mint = ctx.accounts.mint.key();
    config.vault = ctx.accounts.vault.key();

    Ok(())
}

#[derive(Accounts)]
pub struct CreateAndMintTokens<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"stm_config"],
        bump,
        has_one = admin @ CustomError::Unauthorized
    )]
    pub config: Account<'info, Configuration>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 9,
        mint::authority = vault_authority,
        mint::freeze_authority = vault_authority,
        seeds = [b"stm_mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = vault_authority,
        token::token_program = token_program,
        seeds = [b"stm_vault"],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
