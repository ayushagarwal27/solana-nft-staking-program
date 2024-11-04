use anchor_lang::prelude::*;
use anchor_spl::metadata::{Metadata, MasterEditionAccount};
use anchor_spl::metadata::mpl_token_metadata::instructions::{ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts};
use anchor_spl::token::{Token, TokenAccount, Mint, Revoke, revoke};
use crate::error::StakeError;
use crate::state::{StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
      mut,
      associated_token::mint = mint,
      associated_token::authority = user,
    )]
    pub mint_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    #[account(
         mut,
         close = user,
         seeds = [b"stake", config_account.key().as_ref(), mint.key().as_ref()],
         bump = stake_account.bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(
      mut,
     seeds = [b"user".as_ref(), user.key().as_ref()],
     bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
    seeds = [b"config"],
    bump = config_account.bump
    )]
    pub config_account: Account<'info, StakeConfig>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        // check elapsed time
        let time_elapsed = (Clock::get()?.unix_timestamp - self.stake_account.stake_at / 86400) as u32;
        require!(time_elapsed >= self.config_account.freeze_period, StakeError::FreezePeriodNotOver);

        // Increase User points
        self.user_account.points += time_elapsed * self.config_account.points_per_stake as u32;

        // Unfreeze NFT
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();
        let edition = &self.master_edition.to_account_info();

        let seeds = &[
            b"stake",
            self.config_account.to_account_info().key.as_ref(),
            self.mint.to_account_info().key.as_ref(),
            &[self.stake_account.bump]
        ];
        let signer_seeds = &[&seeds[..]];

        ThawDelegatedAccountCpi::new(metadata_program, ThawDelegatedAccountCpiAccounts {
            token_program,
            mint,
            token_account,
            delegate,
            edition,
        }).invoke_signed(signer_seeds)?;

        // Revoke delegation to Stake Account

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Revoke {
            source: self.mint_ata.to_account_info(),
            authority: self.user_account.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        revoke(cpi_context)?;

        // Decrease number of nft staked
        self.user_account.amounts_staked -= 1;
        Ok(())
    }
}