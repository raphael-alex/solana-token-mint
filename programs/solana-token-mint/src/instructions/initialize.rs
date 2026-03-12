use anchor_lang::prelude::*;

use crate::{Configuration, CustomError, MAX_ACCEPTABLE_APY};

// set real pub key
pub const INITIALIZER_KEY: Pubkey = pubkey!("ArFuRYKSMjndfCvs7nLMtHdoN5bVvMeZYsg4EzDZ6S7x");

pub fn initialize_program(ctx: Context<Initialize>, admin: Pubkey, apy: u64) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.admin.key(),
        INITIALIZER_KEY,
        CustomError::Unauthorized
    );

    require!(apy > 0, CustomError::InvalidAPYValue);

    let config = &mut ctx.accounts.config;
    config.admin = admin;
    config.vault_authority = ctx.accounts.vault_authority.key();
    config.bump_vault_authority = ctx.bumps.vault_authority;

    require!(apy <= MAX_ACCEPTABLE_APY, CustomError::InvalidAPYValue);
    config.apy = apy;
    config.pause = false;
    config.version = 1;
    config.total_staked_amount = 0;
    config.last_update_time = Clock::get()?.unix_timestamp;
    config.unstake_percentage_daily = 2000;
    msg!("Project initialized with admin: {}", admin);
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        // init_if_needed,
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Configuration>(),
        seeds = [b"stm_config"],
        bump
    )]
    pub config: Account<'info, Configuration>,
    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
