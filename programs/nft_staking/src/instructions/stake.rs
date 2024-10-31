use anchor_lang::prelude::*;
use anchor_spl::metadata::{MetadataAccount, Metadata, MasterEditionAccount};
use anchor_spl::metadata::mpl_token_metadata::instructions::{FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts};
use anchor_spl::token::{approve, Approve, Mint, Token, TokenAccount};
use crate::error::StakeError;
use crate::state::{StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Stake<'info>{
    #[account(mut)]
    pub user:Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub collection_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub mint_ata: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref()],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata_account.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata_account.collection.as_ref().unwrap().verified == true,
    )]
    pub metadata_account: Account<'info, MetadataAccount>,
    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,
    #[account(
        seeds = [b"config"],
        bump = config_account.bump,
    )]
    pub config_account: Account<'info, StakeConfig>,
    #[account(
        init,
        payer = user,
        seeds = [b"stake", config_account.key().as_ref(), mint.key().as_ref()],
        space = 8 + StakeAccount::INIT_SPACE,
        bump,
    )]
    pub stake_account : Account<'info, StakeAccount>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()>{
        // Delegate Authority of Mint ATA to Stake Account
        require!(self.user_account.amounts_staked <= self.config_account.max_stake, StakeError::MaxStakeReached);

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Approve {
            to: self.mint_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info()
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        approve(cpi_context, 1)?;

        // Freeze Nft
        let delegated_authority = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let master_edition = &self.master_edition.to_account_info();
        let mint  = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();


        let seeds = &[
            b"stake",
            self.config_account.to_account_info().key.as_ref(),
            self.mint.to_account_info().key.as_ref(),
            &[self.stake_account.bump]
        ];
        let signer_seeds = &[&seeds[..]];

        FreezeDelegatedAccountCpi::new(metadata_program, FreezeDelegatedAccountCpiAccounts{
            token_program,
            mint,
            delegate:delegated_authority,
            token_account,
            edition:master_edition
        }).invoke_signed(signer_seeds)?;

        // Initialize Stake Account
        self.stake_account.set_inner(StakeAccount{
            owner: self.user.key(),
            mint: self.mint.key(),
            stake_at:Clock::get()?.unix_timestamp,
            bump: bumps.stake_account
        });

        // Increment user account staked nft amounts
        self.user_account.amounts_staked += 1;
        Ok(())
    }
}