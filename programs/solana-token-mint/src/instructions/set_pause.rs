use anchor_lang::prelude::*;

use crate::{Configuration, CustomError};

pub fn handle_set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.pause = paused;
    msg!("pause status set to : {}", paused);
    Ok(())
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        seeds = [b"stm_config"],
        bump,
        has_one = admin @ CustomError::Unauthorized
    )]
    pub config: Account<'info, Configuration>,
    #[account(mut)]
    pub admin: Signer<'info>,
}
