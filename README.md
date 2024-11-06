### Nft Staking Program

#### Initialize User Instruction

- Creates user account PDA
- User account contains
    - points: reward points
    - amounts_staked: number of nft staked

#### Initialize Config Instruction

- Creates Config PDA
- Config PDA includes
    - points_per_stake: reward points per stake
    - freeze_period: period till which nft needs to be staked
    - max_stake: max number of nft that can be staked
    - rewards_bump: bump of rewards_mint
- Initialize Rewards Mint
- Only Admin can create config and reward_mint

#### Stake Instruction

- Creates Stake PDA
- Stake PDA includes
    - owner: owner of nft
    - mint: mint address of nft
    - stake_at: Unix time stamp when nft was staked
- Delegate Authority of Mint ATA to Stake Account
- Freezes Nft
- Increment user account staked nft by one

#### UnStake Instruction

- Checks elapsed time
- Increases the user reward points
- Unfreezes NFT
- Revokes delegation to Stake Account
- Decreases ft staked number by one

#### Claim Instruction

- Mint reward tokens to User Rewards ATA
- Makes user reward points to zero

Tech Stack : Anchor, Rust

- [Repo Link](https://github.com/ayushagarwal27/anchor-nft-staking-program/tree/main)
