use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use std::vec::Vec;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod defi_os {
    use super::*;

    pub fn initialize_repo(
        ctx: Context<InitializeRepo>,
        _ipfs_hash: String,
        _bump: u8,
        _bump_usdc: u8,
    ) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.repo_vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        mint_to(cpi_ctx, 1000000)?;
        let _repo_account = &mut ctx.accounts.repo_account;
        _repo_account.exchange_token_mint = ctx.accounts.exchange_token_mint.key();
        _repo_account.ipfs_hash = _ipfs_hash;
        _repo_account.vault_bump = _bump;
        _repo_account.vault_bump_usdc = _bump_usdc;
        Ok(())
    }

    pub fn update_repo(ctx: Context<UpdateRepo>, _ipfs_hash: String) -> Result<()> {
        let _repo_account = &mut ctx.accounts.repo_account;
        _repo_account.ipfs_hash = _ipfs_hash;
        Ok(())
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        _tokens: u64,
        _vault_bump: u8,
        // _vault_bump_usdc: u8,
    ) -> Result<()> {
        let _repo_account = &mut ctx.accounts.repo_account;
        require!(
            ctx.accounts.exchange_token_mint.to_account_info().key()
                == _repo_account.exchange_token_mint,
            CustomError::WrongInput
        );

        // let transfer_instruction_1 = anchor_spl::token::Transfer {
        //     from: ctx.accounts.user_exchange_token_account.to_account_info(),
        //     to: ctx.accounts.repo_vault_usdc.to_account_info(),
        //     authority: ctx.accounts.signer.to_account_info(),
        // };

        let transfer_instruction_2 = anchor_spl::token::Transfer {
            from: ctx.accounts.repo_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        // let cpi_ctx_1 = CpiContext::new(
        //     ctx.accounts.token_program.to_account_info(),
        //     transfer_instruction_1,
        // );

        let cpi_ctx_2 = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_2,
        );

        //anchor_spl::token::transfer(cpi_ctx_1, _tokens)?;
        anchor_spl::token::transfer(cpi_ctx_2, _tokens)?;
        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    WrongInput,
    TimeError,
    SameUser,
    WrongUser,
    ChallengeNotExpired,
    ChallengeExpired,
    NoFullConsent,
    NotEnoughFunds,
    VotingAgain,
    LockPeriodNotEnded,
}

#[derive(Accounts)]
pub struct InitializeRepo<'info> {
    #[account(init, payer=signer, space=200)]
    pub repo_account: Account<'info, Repository>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub exchange_token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 9,
        mint::authority = signer,
        mint::freeze_authority = signer,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        seeds = [b"repo-vault".as_ref(),repo_account.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = repo_vault,
    )]
    pub repo_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = signer,
        seeds = [b"repo-treasury".as_ref(),repo_account.key().as_ref()],
        bump,
        token::mint = exchange_token_mint,
        token::authority = repo_vault_usdc,
    )]
    pub repo_vault_usdc: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateRepo<'info> {
    pub repo_account: Account<'info, Repository>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub repo_account: Account<'info, Repository>,
    pub exchange_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"repo-vault".as_ref(),repo_account.key().as_ref()],
        bump = repo_account.vault_bump,
    )]
    pub repo_vault: Account<'info, TokenAccount>,
    // #[account(
    //     mut,
    //     seeds = [b"repo-treasury".as_ref(),repo_account.key().as_ref()],
    //     bump = repo_account.vault_bump_usdc,
    // )]
    // pub repo_vault_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_exchange_token_account.mint ==  exchange_token_mint.key())]
    pub user_exchange_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Repository {
    ipfs_hash: String,
    exchange_token_mint: Pubkey,
    vault_bump: u8,
    vault_bump_usdc: u8,
}
