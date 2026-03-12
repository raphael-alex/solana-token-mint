use crate::config;
use crate::tools;
use crate::Configuration;
use crate::CustomError;
use crate::StakingAccount;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::MintToChecked;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

pub fn handler_stake(ctx: Context<StakingTokens>, amount: u64, compound: bool) -> Result<()> {
    msg!("stake tokens: {}", amount);
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    require!(amount > 0, CustomError::AmountTooSmall);

    require!(
        amount >= config::MIN_STAKE_AMOUNT,
        CustomError::AmountTooSmall
    );

    let config = &mut ctx.accounts.config;
    let stake_acc = &mut ctx.accounts.staking_account;

    tools::update_pool(config)?;

    let mut compounded_reward: u128 = 0;
    if stake_acc.amount.gt(&0) {
        // has stake record
        let reward_accrued_index = config
            .global_reward_per_share
            .checked_sub(stake_acc.user_reward_debt)
            .ok_or(CustomError::ArithmeticOverflow)?;

        let reward_accrued: u128 = (reward_accrued_index
            .checked_mul(stake_acc.amount)
            .ok_or(CustomError::ArithmeticOverflow)?)
        .checked_div(config::PER_SHARE_PRECISION)
        .ok_or(CustomError::ArithmeticOverflow)?;

        if compound {
            compounded_reward = reward_accrued;
        } else {
            stake_acc.claimable_reward = stake_acc
                .claimable_reward
                .checked_add(reward_accrued)
                .ok_or(CustomError::ArithmeticOverflow)?;
        }
    }
    stake_acc.user_reward_debt = config.global_reward_per_share;
    msg!("from ata amount: {}", ctx.accounts.from_ata.amount);
    require!(
        ctx.accounts.from_ata.amount >= amount,
        CustomError::InsufficientTokens
    );

    let staking_accounts = TransferChecked {
        from: ctx.accounts.from_ata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            staking_accounts,
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    let mut total_new_staked: u128 = amount.into();
    if compound && compounded_reward > 0 {
        // mint reward to vault
        let seeds = &[b"vault_authority".as_ref(), &[config.bump_vault_authority]];
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
            compounded_reward as u64,
            ctx.accounts.mint.decimals,
        )?;

        total_new_staked = total_new_staked
            .checked_add(compounded_reward)
            .ok_or(CustomError::ArithmeticOverflow)?;
    }

    stake_acc.owner = ctx.accounts.signer.key();
    stake_acc.bump = ctx.bumps.staking_account;
    stake_acc.amount = stake_acc
        .amount
        .checked_add(total_new_staked)
        .ok_or(CustomError::ArithmeticOverflow)?;
    config.total_staked_amount = config
        .total_staked_amount
        .checked_add(total_new_staked)
        .ok_or(CustomError::ArithmeticOverflow)?;

    tools::update_pool(config)?;

    Ok(())
}

#[derive(Accounts)]
pub struct StakingTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        constraint = from_ata.mint == config.mint
    )]
    pub from_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"stm_vault"],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault_authority"],
        bump = config.bump_vault_authority
    )]
    pub vault_authority: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + std::mem::size_of::<StakingAccount>(),
        seeds = [b"staking_account", signer.key.as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
