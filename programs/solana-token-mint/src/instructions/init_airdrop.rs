use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintToChecked, TokenAccount, TokenInterface},
};

use crate::{AirdropInfo, Configuration, CustomError};

/// init airdrop vault for campaign
/// Airdrop logic:
/// 1. Fetch all historical transaction data from the blockchain explorer via script
/// 2. Filter target users according to rules
/// 3. Generate JSON data file
/// 4. Compress into merkle tree root
/// 5. Store merkle tree root in campaign config
pub fn init_airdrop_campaign(
    ctx: Context<InitAirdrop>,
    campaign_id: u64,
    capacity: u64,
    merkle_root: [u8; 32],
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    msg!("init airdrop campaign status info");
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        start_time < end_time,
        CustomError::StartTimeCannotLaterThanEndTime
    );
    require!(start_time < current_time, CustomError::CampaignOverdue);
    require!(capacity > 0, CustomError::CapacityCannotBeZero);
    let airdrop_info = &mut ctx.accounts.airdrop_info;
    airdrop_info.campaign_id = campaign_id;
    airdrop_info.capacity = capacity;
    airdrop_info.merkle_root = merkle_root;
    airdrop_info.start_time = start_time;
    airdrop_info.end_time = end_time;
    msg!("airdrop start time: {}", airdrop_info.start_time);
    msg!("airdrop end time: {}", airdrop_info.end_time);

    let seeds = &[b"vault_authority".as_ref(), &[ctx.bumps.vault_authority]];
    let mint_cpi_accounts = MintToChecked {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.airdrop_vault.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::mint_to_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_cpi_accounts,
        )
        .with_signer(&[seeds]),
        capacity,
        ctx.accounts.mint.decimals,
    )?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(campaign_id: u64)]
pub struct InitAirdrop<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"stm_config"],
        bump,
        has_one = admin @ CustomError::Unauthorized
    )]
    pub config: Account<'info, Configuration>,

    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: SystemAccount<'info>,

    #[account(
        mut,
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
        seeds = [b"airdrop_vault"],
        bump
    )]
    pub airdrop_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<AirdropInfo>(),
        seeds = [b"airdrop_info", campaign_id.to_le_bytes().as_ref()],
        bump
    )]
    pub airdrop_info: Account<'info, AirdropInfo>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
