use crate::config;
use crate::tools;
use crate::Configuration;
use crate::CustomError;
use crate::StakingAccount;
use crate::MIN_CLAIM_REWARD_THRESHOLD;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintToChecked, TokenAccount, TokenInterface},
};

pub fn handler_reward(ctx: Context<ClaimRewardTokens>) -> Result<()> {
    msg!("Claim reward");

    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    let seeds = &[
        b"vault_authority".as_ref(),
        &[ctx.accounts.config.bump_vault_authority],
    ];

    msg!("claim only reward");
    let config = &mut ctx.accounts.config;
    let stake_acc = &mut ctx.accounts.staking_account;
    require!(stake_acc.amount > 0, CustomError::NoStake);

    tools::update_pool(config)?;

    let total_rewards_to_mint: u64 = tools::calc_total_reward(config, stake_acc)?
        .try_into()
        .map_err(|_| CustomError::ArithmeticOverflow)?;

    require!(
        total_rewards_to_mint > MIN_CLAIM_REWARD_THRESHOLD,
        CustomError::AmountTooSmall
    );
    let current_supply = ctx.accounts.mint.supply;
    require!(
        current_supply.saturating_add(total_rewards_to_mint) <= config::MAX_SUPPLY,
        CustomError::SupplyOverflow
    );
    require!(
        total_rewards_to_mint <= config::MAX_REWARD_PER_CLAIM,
        CustomError::RewardTooHigh
    );

    let mint_to_account = MintToChecked {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::mint_to_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            mint_to_account,
        )
        .with_signer(&[seeds]),
        total_rewards_to_mint,
        ctx.accounts.mint.decimals,
    )?;

    stake_acc.claimable_reward = 0;
    stake_acc.user_reward_debt = config.global_reward_per_share;
    Ok(())
}

// User claims reward tokens as fee payer
#[derive(Accounts)]
pub struct ClaimRewardTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
        seeds = [b"stm_vault"],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + std::mem::size_of::<StakingAccount>(),
        seeds = [b"staking_account", signer.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
