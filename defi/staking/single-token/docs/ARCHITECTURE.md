# Single-Token Staking Architecture

This document describes the architectural design of the Single-Token Staking template.

## Overview

The Single-Token Staking template provides a framework for staking SPL tokens and earning rewards over time. It supports fixed reward rates, stake locking periods, and early withdrawal penalties.

## Core Components

### State Management

- **StakePool**: Stores global staking pool configuration including reward rate, lock period, and statistics.
- **UserStake**: Represents an individual user's stake, including amount, timestamps, and reward tracking.

### Instruction Processing

The template supports six primary operations:

1. **Initialize Pool**: Sets up the staking pool with reward parameters and token accounts.
2. **Stake**: Allows a user to stake tokens for a specified duration.
3. **Unstake**: Enables a user to withdraw staked tokens plus earned rewards.
4. **Claim Rewards**: Permits claiming of rewards without unstaking principal.
5. **Update Pool**: Allows the authority to modify pool parameters.
6. **Fund Rewards**: Adds tokens to the reward pool for distribution.

## Reward Calculation

Rewards are calculated using a time-based formula:

```
rewards = stake_amount * reward_rate * time_elapsed / (10000 * seconds_in_day)
```

Where:
- `reward_rate` is in basis points per day (100 = 1%)
- `time_elapsed` is the time since last claim in seconds
- The denominator normalizes to daily percentages

## Security Considerations

- Authority validation for administrative actions
- Lock period enforcement
- Early withdrawal penalty
- Reward fund availability checks
- Overflow protection for all calculations

## Integration Points

This template interacts with several Solana components:

- **SPL Token Program**: For token transfers
- **System Program**: For account creation
- **Clock Sysvar**: For timestamp validation
- **Rent Sysvar**: For rent exemption validation

## Extension Points

The Single-Token Staking can be extended to support:

- Variable reward rates based on lock duration
- Tiered staking levels
- Compounding rewards
- Multiple token rewards
- Governance integration

## Implementation Notes

- Uses Borsh for efficient serialization/deserialization
- Implements account validation to ensure security
- Tracks staking statistics for analytics
- Handles reward distribution transparently
