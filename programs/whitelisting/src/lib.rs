use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use num_traits::pow;

declare_id!("FGgZC73ai6dw5ynmNUCXbH5FhEeBLqUa6Rx5zTStxFwz");

#[program]
mod token_vault {
    use super::*;
    const COST: u64 = 5;
    const BUY_LIMIT: u64 = 5;
    const DECIMALS: usize = 9;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        require!(!ctx.accounts.whitelist.initialised,
            ErrorCode::AlreadyInitialized);
        
        ctx.accounts.whitelist.whitelist = Vec::new();
        ctx.accounts.whitelist.authority = ctx.accounts.signer.key();
        ctx.accounts.whitelist.initialised =true;
        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
        if !ctx.accounts.whitelist.whitelist.contains(&address) {
            ctx.accounts.whitelist.whitelist.push(address);
            ctx.accounts.counter.tokens = 0
        }
        Ok(())
    }

    pub fn remove_from_whitelist(ctx: Context<ManageWhitelist>, address: Pubkey) -> Result<()> {
        ctx.accounts.whitelist.whitelist.retain(|&x| x != address);
        Ok(())
    }
    //may remove this function
    pub fn transfer_in(ctx: Context<TransferAccounts>, amount: u64) -> Result<()> {
        // Check if the sender is whitelisted
        /* require!(
            ctx.accounts
                .whitelist
                .whitelist
                .contains(&ctx.accounts.signer.key()),
            ErrorCode::NotWhitelisted
        ); */

        msg!("Token amount transfer in: {}!", amount);

        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
    //change to Buy tokens?
    pub fn buy(ctx: Context<TransferAccounts>, amount: u64) -> Result<()> {
        // Check if the recipient is whitelisted
        require!(
            ctx.accounts
                .whitelist
                .whitelist
                .contains(&ctx.accounts.signer.key()),
            ErrorCode::NotWhitelisted
        );
        require!(
            ctx.accounts.counter.tokens + amount <= BUY_LIMIT * pow(10, DECIMALS),
            ErrorCode::BuyLimitExceeded
        );

        msg!("Token amount to purchase: {}!", amount);
        msg!("SOL amount to pay: {}!", COST);

        // Transfer SOL from signer to vault
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                },
            ),
            COST * pow(10, 8) * (amount / pow(10, DECIMALS)),
        )?;
        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.sender_token_account.to_account_info(),
            authority: ctx.accounts.token_account_owner_pda.to_account_info(),
        };

        let bump = ctx.bumps.token_account_owner_pda;
        let seeds = &[b"token_account_owner_pda".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"whitelist"],
        bump,
        space = 8 + 32 + (32 * 100) // Space for up to 100 whitelist addresses
    )]
    pub whitelist: Account<'info, Whitelist>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"token_account_owner_pda"],
        bump,
        space = 8
    )]
    /// CHECK: This is not dangerous because this is just token_owner
    token_account_owner_pda: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"token_vault", mint_of_token_being_sent.key().as_ref()],
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
        bump
    )]
    vault_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ManageWhitelist<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, has_one = authority)] //this may be issue check
    pub whitelist: Account<'info, Whitelist>,
    /// CHECK Only used for whitelisting, which can only be called by owner.
    pub address: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = authority,
        seeds=[address.key().as_ref()],
        bump,
        space = 8 + 32 + (32 * 100) // Space for up to 100 whitelist addresses
    )]
    pub counter: Account<'info, Counter>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferAccounts<'info> {
    pub whitelist: Account<'info, Whitelist>,

    #[account(mut,
        seeds=[b"token_account_owner_pda"],
        bump
    )]
    /// CHECK Address is initialized once by owner, and remains unchanged after that.
    token_account_owner_pda: AccountInfo<'info>,

    #[account(mut,
        seeds=[b"token_vault", mint_of_token_being_sent.key().as_ref()],
        bump,
        token::mint=mint_of_token_being_sent,
        token::authority=token_account_owner_pda,
    )]
    vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    sender_token_account: Account<'info, TokenAccount>,
    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    #[account(mut,
        seeds=[signer.key().as_ref()],
        bump,
        )]
    counter: Account<'info, Counter>,
}

#[account]
pub struct Whitelist {
    pub whitelist: Vec<Pubkey>,
    pub authority: Pubkey,
    pub initialised:bool,
}
#[account]
pub struct Counter {
    pub tokens: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Address is not whitelisted")]
    NotWhitelisted,
    #[msg("Address will exceed buy limit")]
    BuyLimitExceeded,
    #[msg("Accounts already initialized")]
    AlreadyInitialized,
}
