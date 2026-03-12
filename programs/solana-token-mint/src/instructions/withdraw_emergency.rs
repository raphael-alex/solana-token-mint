use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{AirdropInfo, Configuration, CustomError};

pub fn handler_emergency(ctx: Context<EmergencyWithdrawTokens>, campaign_id: u64) -> Result<()> {
    msg!(
        "Withdraw tokens in capacity for campaign_id: {} by an emergency",
        campaign_id
    );
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    msg!(
        "airdrop_info capacity: {}",
        ctx.accounts.airdrop_info.capacity
    );
    msg!(
        "airdrop_vault amount: {}",
        ctx.accounts.airdrop_vault.amount
    );
    msg!("config apy: {}", ctx.accounts.config.apy);
    require!(
        0 < ctx.accounts.airdrop_info.capacity,
        CustomError::InsufficientAirdropAmount,
    );

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[ctx.accounts.config.bump_vault_authority],
    ]];
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.airdrop_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_cpi_accounts,
        )
        .with_signer(signer_seeds),
        ctx.accounts.airdrop_info.capacity,
        ctx.accounts.mint.decimals,
    )?;
    ctx.accounts.airdrop_info.capacity = 0;
    Ok(())
}

// push token to users
#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct EmergencyWithdrawTokens<'info> {
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
        seeds = [b"airdrop_info", campaign_id.to_le_bytes().as_ref()],
        bump
    )]
    pub airdrop_info: Account<'info, AirdropInfo>,
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
    #[account(
        mut,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"airdrop_vault"],
        bump
    )]
    pub airdrop_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
