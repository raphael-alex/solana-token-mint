use anchor_lang::prelude::*;

use crate::{Configuration, CustomError, ADMIN_TRANSFER_DELAY};

/// Initiate admin transfer request
/// Only the current admin can call this, sets the pending admin and timestamp
pub fn handle_propose_admin_transfer(
    ctx: Context<ProposeAdminTransfer>,
    new_admin: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Check if there is already a pending transfer request
    require!(
        config.pending_admin.is_none(),
        CustomError::AdminTransferAlreadyInProgress
    );

    // New admin cannot be the current admin
    require!(
        config.admin != new_admin,
        CustomError::AdminCannotBeRecipient
    );

    let current_time = Clock::get()?.unix_timestamp;

    config.pending_admin = Some(new_admin);
    config.admin_transfer_initiated_at = Some(current_time);

    msg!(
        "Admin transfer proposed. Current admin: {}, New admin: {}, Time: {}",
        config.admin,
        new_admin,
        current_time
    );
    msg!(
        "Transfer can be confirmed after {} seconds (at timestamp {})",
        ADMIN_TRANSFER_DELAY,
        current_time + ADMIN_TRANSFER_DELAY
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ProposeAdminTransfer<'info> {
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

/// Confirm admin transfer
/// Only the pending admin can call this, and must wait for the delay period
pub fn handle_confirm_admin_transfer(ctx: Context<ConfirmAdminTransfer>) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Check if there is a pending transfer request
    let pending_admin = config
        .pending_admin
        .ok_or(CustomError::NoPendingAdminTransfer)?;

    let initiated_at = config
        .admin_transfer_initiated_at
        .ok_or(CustomError::NoPendingAdminTransfer)?;

    // Verify the caller is the pending admin
    require!(
        ctx.accounts.new_admin.key() == pending_admin,
        CustomError::OnlyPendingAdminCanConfirm
    );

    // Verify the delay period has passed
    let current_time = Clock::get()?.unix_timestamp;
    let elapsed = current_time - initiated_at;
    require!(
        elapsed >= ADMIN_TRANSFER_DELAY,
        CustomError::AdminTransferDelayNotMet
    );

    // Execute the transfer
    let old_admin = config.admin;
    config.admin = pending_admin;
    config.pending_admin = None;
    config.admin_transfer_initiated_at = None;

    msg!(
        "Admin transfer confirmed. Old admin: {}, New admin: {}",
        old_admin,
        config.admin
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ConfirmAdminTransfer<'info> {
    #[account(
        mut,
        seeds = [b"stm_config"],
        bump,
    )]
    pub config: Account<'info, Configuration>,
    #[account(mut)]
    pub new_admin: Signer<'info>,
}

/// Cancel admin transfer request
/// Only the current admin can call this
pub fn handle_cancel_admin_transfer(ctx: Context<CancelAdminTransfer>) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Check if there is a pending transfer request
    require!(
        config.pending_admin.is_some(),
        CustomError::NoPendingAdminTransfer
    );

    let cancelled_admin = config.pending_admin.unwrap();
    config.pending_admin = None;
    config.admin_transfer_initiated_at = None;

    msg!(
        "Admin transfer cancelled. Cancelled pending admin: {}",
        cancelled_admin
    );

    Ok(())
}

#[derive(Accounts)]
pub struct CancelAdminTransfer<'info> {
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
