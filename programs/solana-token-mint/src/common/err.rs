use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("The amount transferred can not smaller than 0.0000002.")]
    AmountTooSmall,
    #[msg("The amount transferred can not be zero.")]
    AmountCannotBeZero,
    #[msg("Arithmetic overflow occurred.")]
    ArithmeticOverflow,
    #[msg("Unauthorized action.")]
    Unauthorized,
    #[msg("Insufficient staked amount.")]
    InsufficientStakedAmount,
    #[msg("Stake Time is wrong.")]
    WrongTime,
    #[msg("No Stake.")]
    NoStake,
    #[msg("No reward.")]
    NoReward,
    #[msg("Excessive initial supply")]
    ExcessiveInitialSupply,
    #[msg("Excessive issuance")]
    ExcessiveIssuance,
    #[msg("Invalid apy value")]
    InvalidAPYValue,
    #[msg("Supply overflow")]
    SupplyOverflow,
    #[msg("Not on the airdrop list")]
    NotOnAirdropList,
    #[msg("Insufficient airdrop amount")]
    InsufficientAirdropAmount,
    #[msg("Airdrop not started")]
    AirdropNotStarted,
    #[msg("Overdue airdrop")]
    OverdueAirdrop,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Reward amount too high")]
    RewardTooHigh,
    #[msg("Insufficient tokens")]
    InsufficientTokens,
    #[msg("Exceed unstake limit")]
    ExceedUnstakeLimit,
    #[msg("Admin cannot be recipient")]
    AdminCannotBeRecipient,
    #[msg("Pause by an emergency")]
    EmergencyPause,
    #[msg("Capacity can not be zero")]
    CapacityCannotBeZero,
    #[msg("Start time can not later than end time")]
    StartTimeCannotLaterThanEndTime,
    #[msg("Campaign overdue")]
    CampaignOverdue,
    #[msg("No pending admin transfer")]
    NoPendingAdminTransfer,
    #[msg("Admin transfer delay not met")]
    AdminTransferDelayNotMet,
    #[msg("Only pending admin can confirm transfer")]
    OnlyPendingAdminCanConfirm,
    #[msg("Admin transfer already in progress")]
    AdminTransferAlreadyInProgress,
    #[msg("Admin transfer cancelled")]
    AdminTransferCancelled,
}
