use crate::AirdropInfo;
use crate::ClaimStatus;
use crate::Configuration;
use crate::CustomError;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account::solana_program::keccak;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

pub fn handler_airdrop(
    ctx: Context<ClaimAirdropTokens>,
    campaign_id: u64,
    amount: u64,
    merkle_proof: Vec<[u8; 32]>,
) -> Result<()> {
    msg!(
        "Claim airdrop tokens: {}, campaign id is : {}",
        amount,
        campaign_id
    );

    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    msg!(
        "airdrop start time: {}",
        ctx.accounts.airdrop_info.start_time
    );
    msg!("airdrop end time: {}", ctx.accounts.airdrop_info.end_time);
    let current_time = Clock::get()?.unix_timestamp;
    msg!("current time: {}", current_time);
    require!(
        ctx.accounts.airdrop_info.start_time <= current_time,
        CustomError::AirdropNotStarted,
    );
    require!(
        ctx.accounts.airdrop_info.end_time >= current_time,
        CustomError::OverdueAirdrop,
    );

    let user_key = ctx.accounts.signer.key();
    let leaf = keccak::hashv(&[
        &user_key.to_bytes(),
        &campaign_id.to_le_bytes(),
        &amount.to_le_bytes(),
    ])
    .0;
    let mut current_hash = leaf;
    for node in merkle_proof {
        if current_hash <= node {
            current_hash = keccak::hashv(&[&current_hash, &node]).0;
        } else {
            current_hash = keccak::hashv(&[&node, &current_hash]).0;
        }
    }

    require!(
        current_hash == ctx.accounts.airdrop_info.merkle_root,
        CustomError::NotOnAirdropList
    );

    let seeds = &[
        b"vault_authority".as_ref(),
        &[ctx.accounts.config.bump_vault_authority],
    ];

    require!(amount > 0, CustomError::AmountCannotBeZero);
    require!(
        amount <= ctx.accounts.airdrop_info.capacity,
        CustomError::InsufficientAirdropAmount,
    );
    require!(
        ctx.accounts.airdrop_vault.amount >= amount,
        CustomError::InsufficientVaultBalance
    );

    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.airdrop_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_cpi_accounts,
        )
        .with_signer(&[seeds]),
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

// A user gets Airdrop tokens as a fee payer
#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct ClaimAirdropTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + std::mem::size_of::<ClaimStatus>(),
        seeds = [
            b"claim_status",
            campaign_id.to_le_bytes().as_ref(),
            signer.key().as_ref()
        ],
        bump
    )]
    pub claim_status: Account<'info, ClaimStatus>,
    #[account(
        mut,
        seeds = [b"airdrop_info", campaign_id.to_le_bytes().as_ref()],
        bump
    )]
    pub airdrop_info: Account<'info, AirdropInfo>,
    #[account(
        seeds = [b"stm_config"],
        bump
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
        seeds = [b"airdrop_vault"],
        bump
    )]
    pub airdrop_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
