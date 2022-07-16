use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
const COMPANY_SEED: &'static [u8] = b"company";
const POOL_SEED: &'static [u8] = b"pool";

#[program]
pub mod token_transfer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _company_id: String) -> Result<()> {
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        company_id: String,
        base_bump: u8,
        _pool_bump: u8,
        amount: u64,
    ) -> Result<()> {
        let parameters = &mut ctx.accounts.base_account;

        let bump_vector = base_bump.to_le_bytes();
        let inner = vec![
            COMPANY_SEED,
            company_id.as_bytes()[..18].as_ref(),
            company_id.as_bytes()[18..].as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
            to: ctx.accounts.pool_wallet.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(), //signer PDA
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        parameters.staked_amount = amount;

        Ok(())
    }

    pub fn transfer(
        ctx: Context<TransferAmount>,
        company_id: String,
        base_bump: u8,
        _pool_bump: u8,
        amount: u64,
    ) -> Result<()> {
        let parameters = &mut ctx.accounts.base_account;

        if amount < parameters.staked_amount {
            let bump_vector = base_bump.to_le_bytes();
            let inner = vec![
                COMPANY_SEED,
                company_id.as_bytes()[..18].as_ref(),
                company_id.as_bytes()[18..].as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];

            let transfer_instruction = Transfer {
                from: ctx.accounts.pool_wallet.to_account_info(),
                to: ctx.accounts.wallet_to_deposit_to.to_account_info(),
                authority: parameters.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
                outer.as_slice(), //signer PDA
            );

            anchor_spl::token::transfer(cpi_ctx, amount)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(company_id: String)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, seeds = [COMPANY_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()], bump, space = 100)]
    pub base_account: Account<'info, Parameters>,
    #[account(init, payer = authority, seeds = [POOL_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()], bump, token::mint=token_mint,
    token::authority=base_account)]
    pub pool_wallet: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(company_id: String, base_bump: u8, pool_bump: u8)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [COMPANY_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()], bump = base_bump)]
    pub base_account: Account<'info, Parameters>,
    #[account(
        mut,
        seeds = [POOL_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()],
        bump = pool_bump,
        token::mint=token_mint,
        token::authority=base_account,
    )]
    pub pool_wallet: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub wallet_to_withdraw_from: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(company_id: String, base_bump: u8, pool_bump: u8)]
pub struct TransferAmount<'info> {
    #[account(mut, seeds = [COMPANY_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()], bump = base_bump)]
    pub base_account: Account<'info, Parameters>,
    #[account(
        mut,
        seeds = [POOL_SEED, company_id.as_bytes()[..18].as_ref(), company_id.as_bytes()[18..].as_ref()],
        bump = pool_bump,
        token::mint=token_mint,
        token::authority=base_account,
    )]
    pub pool_wallet: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub wallet_to_deposit_to: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Parameters {
    pub staked_amount: u64,
}
