use crate::{
    config,
    tools::{self},
    Configuration, CustomError, StakingAccount,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, MintToChecked, TokenAccount, TokenInterface, TransferChecked},
};

pub fn handler_unstake(ctx: Context<UnstakeTokens>, amount: u64, reward: bool) -> Result<()> {
    msg!("Unstake tokens: {}", amount);
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vault_authority",
        &[ctx.accounts.config.bump_vault_authority],
    ]];
    let config = &mut ctx.accounts.config;
    let stake_acc = &mut ctx.accounts.staking_account;
    require!(
        stake_acc.amount >= amount.into(),
        CustomError::InsufficientStakedAmount
    );

    let now = Clock::get()?.unix_timestamp;
    let mut unstaked_in_24_hours = stake_acc.unstaked_in_24_hours;
    msg!("unstake in 24 hours: {}", unstaked_in_24_hours);
    if let Some(last_unstake_time) = stake_acc.last_unstake_time {
        let interval = now - last_unstake_time;
        if interval >= config::SECONDS_PER_DAY {
            unstaked_in_24_hours = 0;
        }
    }
    let last_unstake_in_24_hours = unstaked_in_24_hours + amount;
    let unstake_percentage_daily_u128 = u128::from(config.unstake_percentage_daily);
    let unstake_limit_u128 = stake_acc
        .amount
        .checked_mul(unstake_percentage_daily_u128)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_div(config::UNSTAKE_PRECISION)
        .ok_or(CustomError::ArithmeticOverflow)?;
    let unstake_limit = u64::try_from(unstake_limit_u128).map_err(|e| {
        msg!("Error converting unstake limit to u64: {:?}", e);
        CustomError::ArithmeticOverflow
    })?;

    msg!("unstake_limit: {:?}", unstake_limit);
    msg!("last unstake in 24 hours: {:?}", last_unstake_in_24_hours);
    require!(
        amount <= unstake_limit && last_unstake_in_24_hours <= unstake_limit,
        CustomError::ExceedUnstakeLimit
    );

    tools::update_pool(config)?;

    let total_reward_to_mint: u64 = tools::calc_total_reward(config, stake_acc)?
        .try_into()
        .map_err(|_| CustomError::ArithmeticOverflow)?;

    if total_reward_to_mint > 0 && reward {
        let current_supply = ctx.accounts.mint.supply;
        require!(
            current_supply.saturating_add(total_reward_to_mint) <= config::MAX_SUPPLY,
            CustomError::SupplyOverflow
        );

        let reward_mint_account = MintToChecked {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to_ata.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        };

        token_interface::mint_to_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                reward_mint_account,
            )
            .with_signer(signer_seeds),
            total_reward_to_mint,
            ctx.accounts.mint.decimals,
        )?;
    }

    let unstaking_accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.to_ata.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            unstaking_accounts,
        )
        .with_signer(signer_seeds),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    stake_acc.amount = stake_acc
        .amount
        .checked_sub(amount.into())
        .ok_or(CustomError::ArithmeticOverflow)?;
    stake_acc.user_reward_debt = config.global_reward_per_share;
    stake_acc.last_unstake_time = Some(now);
    stake_acc.unstaked_in_24_hours = last_unstake_in_24_hours;
    if reward {
        stake_acc.claimable_reward = 0;
    }
    config.total_staked_amount = config
        .total_staked_amount
        .checked_sub(amount.into())
        .ok_or(CustomError::ArithmeticOverflow)?;
    tools::update_pool(config)?;
    Ok(())
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [b"stm_config"],
        bump
    )]
    pub config: Account<'info, Configuration>,

    #[account(
        seeds = [b"stm_mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

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
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        constraint = to_ata.mint == config.mint
    )]
    pub to_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"staking_account", signer.key().as_ref()],
        bump = staking_account.bump,
        constraint = staking_account.owner == signer.key() @ CustomError::Unauthorized,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
