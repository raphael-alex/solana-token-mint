#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
pub mod instructions;
pub use instructions::*;
pub mod common;
pub use common::*;

declare_id!("6rr32NF3a3g5jxao3JP7PKqZe2Lpc43LEtzDFkpLDe5Q");

#[program]
pub mod stm_token_mint_program {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey, apy: u64) -> Result<()> {
        initialize::initialize_program(ctx, admin, apy)
    }

    pub fn init_airdrop(
        ctx: Context<InitAirdrop>,
        campaign_id: u64,
        capacity: u64,
        merkle_root: [u8; 32],
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        init_airdrop::init_airdrop_campaign(
            ctx,
            campaign_id,
            capacity,
            merkle_root,
            start_time,
            end_time,
        )
    }

    pub fn create_and_mint_tokens(ctx: Context<CreateAndMintTokens>, amount: u64) -> Result<()> {
        create_mint::handler_mint(ctx, amount)
    }

    pub fn withdraw_tokens(
        ctx: Context<WithdrawTokens>,
        campaign_id: u64,
        amount: u64,
    ) -> Result<()> {
        withdraw_airdrop::handler_withdraw(ctx, campaign_id, amount)
    }

    pub fn withdraw_emergency(
        ctx: Context<EmergencyWithdrawTokens>,
        campaign_id: u64,
    ) -> Result<()> {
        withdraw_emergency::handler_emergency(ctx, campaign_id)
    }

    pub fn claim_aridrop(
        ctx: Context<ClaimAirdropTokens>,
        campaign_id: u64,
        amount: u64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        claim_aridrop::handler_airdrop(ctx, campaign_id, amount, merkle_proof)
    }

    pub fn transfer_token(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        transfer::handler_transfer(ctx, amount)
    }

    // Increase issuance
    pub fn increase_issuance(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        increase_issuance::handler_increase_issuance(ctx, amount)
    }

    // Staking
    pub fn stake(ctx: Context<StakingTokens>, amount: u64, compound: bool) -> Result<()> {
        stake::handler_stake(ctx, amount, compound)
    }

    pub fn unstake(ctx: Context<UnstakeTokens>, amount: u64, reward: bool) -> Result<()> {
        unstake::handler_unstake(ctx, amount, reward)
    }

    // Burn
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        burn_tokens::handler_burn_tokens(ctx, amount)
    }

    // Time-locked admin transfer
    pub fn propose_admin_transfer(
        ctx: Context<ProposeAdminTransfer>,
        new_admin: Pubkey,
    ) -> Result<()> {
        update_admin::handle_propose_admin_transfer(ctx, new_admin)
    }

    pub fn confirm_admin_transfer(ctx: Context<ConfirmAdminTransfer>) -> Result<()> {
        update_admin::handle_confirm_admin_transfer(ctx)
    }

    pub fn cancel_admin_transfer(ctx: Context<CancelAdminTransfer>) -> Result<()> {
        update_admin::handle_cancel_admin_transfer(ctx)
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        set_pause::handle_set_pause(ctx, paused)
    }
}
