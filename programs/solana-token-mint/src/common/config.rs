use anchor_lang::prelude::*;

pub const SECONDS_PER_YEAR: i64 = 31_536_000;
pub const SECONDS_PER_DAY: i64 = 86_400;
pub const APY_PRECISION: u128 = 1_000_000_000;
pub const PER_SHARE_PRECISION: u128 = 1_000_000_000_000_000_000;
pub const FEE_PRECISION: u128 = 10_000;
pub const UNSTAKE_PRECISION: u128 = 10_000;
pub const MAX_INITIAL_SUPPLY: u64 = 1_000_000_000_000_000_000;
pub const MAX_ISSUANCE_PER_CALL: u64 = 1_000_000_000_000_000_000;
pub const MAX_ACCEPTABLE_APY: u64 = 500_000_000;
pub const MIN_CLAIM_REWARD_THRESHOLD: u64 = 1_000_000_000;
pub const MIN_STAKE_AMOUNT: u64 = 10_000_000_000;
pub const MAX_SUPPLY: u64 = 9_000_000_000_000_000_000;
pub const MAX_REWARD_PER_CLAIM: u64 = 1_000_000_000_000;

// Time-lock configuration
pub const ADMIN_TRANSFER_DELAY: i64 = 48 * 60 * 60; // 48-hour delay (seconds)

#[account]
pub struct StakingAccount {
    pub owner: Pubkey,
    pub amount: u128,           // Staked principal
    pub user_reward_debt: u128, // User claim anchor, i.e., GRPS value at last settlement
    pub claimable_reward: u128, // Amount of tokens settled but not yet withdrawn by user
    pub bump: u8,
    pub last_unstake_time: Option<i64>,
    pub unstaked_in_24_hours: u64,
}

#[account]
pub struct Configuration {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub vault_authority: Pubkey,
    pub bump_vault_authority: u8,
    pub version: u8,
    pub pause: bool, // Paused
    pub last_migration_ts: u64,
    // Time-lock fields - for admin transfer
    pub pending_admin: Option<Pubkey>,      // Pending new admin
    pub admin_transfer_initiated_at: Option<i64>, // Transfer request initiation time
    pub _reserved: [u8; 8],  // Reserved space
    pub apy: u64,                      // Annual percentage yield
    pub global_reward_per_share: u128, // Global reward index
    pub last_update_time: i64,         // Timestamp of last GRPS update
    pub total_staked_amount: u128,     // Total staked tokens in the pool
    pub fee_percentage: u64,
    pub vault_fee_percentage: u64,
    pub unstake_percentage_daily: u64,
}

#[account]
pub struct ClaimStatus {
    pub campaign_id: u64,
    pub claimed: bool,
    pub bump: u8,
}

#[account]
pub struct AirdropInfo {
    pub campaign_id: u64,
    pub capacity: u64,
    pub merkle_root: [u8; 32],
    pub start_time: i64,
    pub end_time: i64,
}
