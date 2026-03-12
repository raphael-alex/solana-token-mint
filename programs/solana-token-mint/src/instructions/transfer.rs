use crate::{config, Configuration, CustomError};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, BurnChecked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

const MINIMUM_LAMPORTS_PER_TRANSFER: u64 = 2000;

pub fn handler_transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
    msg!("Greetings from: {:?}", ctx.program_id);
    require!(!ctx.accounts.config.pause, CustomError::EmergencyPause);
    require!(
        amount >= MINIMUM_LAMPORTS_PER_TRANSFER,
        CustomError::AmountTooSmall
    );

    let fee_percentage = u128::from(ctx.accounts.config.fee_percentage); // Assuming this is set in config as basis points (e.g., 100 = 1%)
    let amount_u128 = u128::from(amount);
    let fee = amount_u128
        .checked_mul(fee_percentage)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_div(config::FEE_PRECISION) // 10,000 basis points = 100%
        .ok_or(CustomError::ArithmeticOverflow)?;

    let vault_fee_percentage = u128::from(ctx.accounts.config.vault_fee_percentage); // Assuming this is configurable (e.g., 50 for 50%)
    let vault_fee = fee
        .checked_mul(vault_fee_percentage)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_div(config::FEE_PRECISION)
        .ok_or(CustomError::ArithmeticOverflow)?; // This is how much goes to vault
    let burn_fee = fee - vault_fee; // This is how much gets burned

    // Transfer to the receiver (amount - fee)
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.sender_ata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_ata.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_cpi_accounts,
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;

    // Burn half of the fee (fee/2)
    let burn_cpi_accounts = BurnChecked {
        from: ctx.accounts.sender_ata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let burn_fee_u64 = u64::try_from(burn_fee).map_err(|e| {
        msg!("Error converting burn_fee to u64: {:?}", e);
        CustomError::ArithmeticOverflow
    })?;
    token_interface::burn_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            burn_cpi_accounts,
        ),
        burn_fee_u64,
        ctx.accounts.mint.decimals,
    )?;

    let vault_fee_u64 = u64::try_from(vault_fee).map_err(|e| {
        msg!("Error converting vault_fee to u64: {:?}", e);
        CustomError::ArithmeticOverflow
    })?;
    // Transfer to the project account (fee/2)
    let earn_cpi_accounts = TransferChecked {
        from: ctx.accounts.sender_ata.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            earn_cpi_accounts,
        ),
        vault_fee_u64,
        ctx.accounts.mint.decimals,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub fee_payer: Signer<'info>,
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub sender_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = fee_payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,
    pub recipient: SystemAccount<'info>,
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
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
