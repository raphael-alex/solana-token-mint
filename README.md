# Solana Token Mint Program

A Solana program built with Anchor framework (v0.32.1) for token minting, staking with APY rewards, and Merkle tree-based airdrop functionality.

**Program ID:** `6rr32NF3a3g5jxao3JP7PKqZe2Lpc43LEtzDFkpLDe5Q`

## Features

- **Token Minting**: Create and mint SPL tokens with controlled supply
- **Staking System**: Stake tokens with APY-based rewards calculation
- **Airdrop Distribution**: Merkle tree-based airdrop claims for gas-efficient token distribution
- **Token Operations**: Transfer, burn, and increase token supply
- **Admin Management**: Time-locked admin transfer for enhanced security
- **Pause Mechanism**: Emergency pause functionality

## Prerequisites

- Rust 1.75+
- Solana CLI 1.18+
- Anchor CLI 0.32.1
- Node.js 18+
- Yarn

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd solana-token-mint

# Install dependencies
yarn install
```

## Build and Test

```bash
# Build the program
anchor build

# Run all tests (localnet)
anchor test

# Run tests without rebuilding
anchor test --skip-build

# Deploy to configured cluster
anchor deploy

# Lint TypeScript files
yarn lint

# Fix linting issues
yarn lint:fix
```

## Architecture

### Instructions

| Instruction              | Description                                         |
| ------------------------ | --------------------------------------------------- |
| `initialize`             | Initialize program configuration with admin and APY |
| `create_and_mint_tokens` | Create mint and initial token supply                |
| `stake`                  | Stake tokens with optional compounding              |
| `unstake`                | Unstake tokens with daily limits                    |
| `claim_aridrop`          | Claim airdrop tokens via Merkle proof verification  |
| `init_airdrop`           | Initialize airdrop campaign with Merkle root        |
| `transfer_token`         | Transfer tokens between accounts                    |
| `burn_tokens`            | Burn tokens from user account                       |
| `increase_issuance`      | Mint additional tokens (admin only)                 |
| `propose_admin_transfer` | Propose new admin with 48h time-lock                |
| `confirm_admin_transfer` | Confirm admin transfer after delay                  |
| `set_pause`              | Pause/unpause program operations                    |

### PDA Seeds

| Seed                                | Purpose                            |
| ----------------------------------- | ---------------------------------- |
| `stm_config`                        | Global configuration account       |
| `stm_mint`                          | Token mint PDA                     |
| `stm_vault`                         | Main staking vault                 |
| `vault_authority`                   | Authority PDA for vault signatures |
| `staking_account` + user pubkey     | User's staking state               |
| `airdrop_info` + campaign_id        | Airdrop campaign details           |
| `airdrop_vault`                     | Tokens reserved for airdrop        |
| `claim_status` + campaign_id + user | Tracks airdrop claims              |

### Staking Rewards Mechanism

The staking system uses a **Global Reward Per Share (GRPS)** model for efficient reward calculation:

- **GRPS**: Accrued value tracking rewards per staked token
- **User Reward Debt**: Snapshot of GRPS when user last interacted
- **Reward Formula**: `reward = (currentGRPS - userRewardDebt) * stakedAmount / PER_SHARE_PRECISION`

Pool updates via `tools::update_pool()` are called before any stake/unstake operation.

### Airdrop Verification

Uses on-chain Keccak256 for Merkle proof verification:

- **Leaf format**: `keccak256(user_address || campaign_id || amount)`
- **Proof verification**: Sorted pairs for deterministic proof generation

## Constants

| Constant               | Value                     | Description                   |
| ---------------------- | ------------------------- | ----------------------------- |
| `APY_PRECISION`        | 1,000,000,000             | APY calculation precision     |
| `PER_SHARE_PRECISION`  | 1,000,000,000,000,000,000 | Reward per share precision    |
| `MAX_SUPPLY`           | 9,000,000,000,000,000,000 | 9 billion tokens (9 decimals) |
| `MIN_STAKE_AMOUNT`     | 10,000,000,000            | 10 tokens minimum stake       |
| `MAX_ACCEPTABLE_APY`   | 500,000,000               | Maximum APY limit             |
| `ADMIN_TRANSFER_DELAY` | 172,800                   | 48 hours (seconds)            |

## File Structure

```
programs/solana-token-mint/src/
├── lib.rs              # Program entrypoint and instruction routing
├── instructions/       # Individual instruction handlers
│   ├── initialize.rs   # Program initialization
│   ├── stake.rs        # Stake tokens with optional compounding
│   ├── unstake.rs      # Unstake with daily limits
│   ├── create_mint.rs  # Create mint and initial supply
│   ├── claim_aridrop.rs # Merkle proof verification and claim
│   ├── init_airdrop.rs # Initialize airdrop campaign
│   ├── transfer.rs     # Token transfer
│   ├── burn_tokens.rs  # Burn tokens
│   ├── increase_issuance.rs # Mint additional tokens
│   ├── update_admin.rs # Admin transfer with time-lock
│   ├── set_pause.rs    # Pause/unpause operations
│   ├── withdraw_airdrop.rs # Withdraw unclaimed airdrop tokens
│   └── withdraw_emergency.rs # Emergency withdrawal
└── common/
    ├── config.rs       # Account structs and constants
    ├── err.rs          # Custom error codes
    ├── mod.rs          # Module exports
    └── tools.rs        # Reward calculation utilities
```

## Testing

Tests use `anchor.workspace` to access the program and run on localnet with a configured timeout of 1,000,000ms.

Key test setup includes:

- Deriving PDAs with `PublicKey.findProgramAddressSync`
- Building Merkle tree with `merkletreejs` and `keccak256`
- Airdropping SOL to test users before operations

## Token Program Support

The program uses `TokenInterface` (not `Token`) to support both SPL Token and Token-2022 programs. Token operations use `token_interface::transfer_checked` for compatibility.

## Security Features

1. **Time-locked Admin Transfer**: 48-hour delay before admin transfer can be confirmed
2. **Pause Mechanism**: Emergency pause for all program operations
3. **Daily Unstake Limits**: Configurable percentage-based unstake limits
4. **Merkle Proof Verification**: On-chain verification for airdrop claims

## License

MIT
