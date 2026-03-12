use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{AirdropInfo, ClaimStatus, Configuration, CustomError};

pub fn handler_withdraw(ctx: Context<WithdrawTokens>, campaign_id: u64, amount: u64) -> Result<()> {
    msg!(
        "Withdraw {} tokens for campaign_id: {}",
        amount,
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
    let current_time = Clock::get()?.unix_timestamp;
    if !ctx.accounts.recipient.key().eq(&ctx.accounts.admin.key()) {
        require!(
            ctx.accounts.airdrop_info.start_time <= current_time,
            CustomError::AirdropNotStarted,
        );
        require!(
            ctx.accounts.airdrop_info.end_time >= current_time,
            CustomError::OverdueAirdrop,
        );
    }
    require!(amount > 0, CustomError::AmountCannotBeZero);
    require!(
        amount <= ctx.accounts.airdrop_info.capacity,
        CustomError::InsufficientAirdropAmount,
    );
    require!(
        ctx.accounts.airdrop_vault.amount >= amount,
        CustomError::InsufficientVaultBalance
    );
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[ctx.accounts.config.bump_vault_authority],
    ]];
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.airdrop_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_ata.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_cpi_accounts,
        )
        .with_signer(signer_seeds),
        amount,
        ctx.accounts.mint.decimals,
    )?;
    ctx.accounts.claim_status.claimed = true;
    ctx.accounts.claim_status.campaign_id = campaign_id;
    ctx.accounts.claim_status.bump = ctx.bumps.claim_status;
    let lastest_capacity = ctx.accounts.airdrop_info.capacity - amount;
    ctx.accounts.airdrop_info.capacity = lastest_capacity;
    Ok(())
}

// push token to users
#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct WithdrawTokens<'info> {
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
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<ClaimStatus>(),
        seeds = [
            b"claim_status",
            campaign_id.to_le_bytes().as_ref(),
            recipient.key().as_ref()
        ],
        bump
    )]
    pub claim_status: Account<'info, ClaimStatus>,
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
        seeds = [b"airdrop_vault"],
        bump
    )]
    pub airdrop_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        constraint = recipient.key() != admin.key() @ CustomError::AdminCannotBeRecipient,
    )]
    pub recipient: SystemAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
