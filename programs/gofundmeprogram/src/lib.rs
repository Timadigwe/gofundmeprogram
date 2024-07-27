use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

#[constant]
pub const OWNER_PREFIX: &[u8] = b"owner";

#[constant]
pub const VAULT_PREFIX: &[u8] = b"vault";
declare_id!("5L1hGNy2PwsE1WMzyALoZtMtFnf2wf7swMW7BYmckfEC");

#[program]
pub mod gofundmeprogram {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, campaign_name: String) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        campaign.owner = *ctx.accounts.user.key;
        campaign.name = campaign_name;
        campaign.amount_raised = 0;
        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, campaign_name: String, amount: u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

        campaign.amount_raised += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Donate>, campaign_name: String, amount: u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let owner = &ctx.accounts.user;

        if campaign.owner != *owner.key {
            return Err(ErrorCode::Unauthorized.into());
        }

        if campaign.amount_raised < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // **owner.to_account_info().try_borrow_mut_lamports()? -= amount;

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: campaign.to_account_info(),
        };

        //let signer = ctx.accounts.user.key;
        let bump = &[ctx.bumps.campaign];
        let campaign_name = campaign.name.clone();
        let signer = &[&[OWNER_PREFIX, campaign_name.as_bytes(), bump][..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );

        anchor_spl::token::transfer(cpi_ctx, ctx.accounts.vault_token_account.amount)?;

        campaign.amount_raised -= amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(campaign_name: String)]
pub struct Initialize<'info> {
    #[account(init,
         payer = user,
         seeds=[OWNER_PREFIX, campaign_name.as_ref()],
         bump,
         space = 8 + 32 + 40 + 8
    )]
    campaign: Account<'info, Campaign>,

    #[account(
            init,
            payer = user,
            seeds=[VAULT_PREFIX,mint_of_token_being_sent.key().as_ref(), campaign_name.as_ref()],
            token::mint = mint_of_token_being_sent,
            token::authority = campaign,
            bump
        )]
    vault_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    user: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(campaign_name: String)]
pub struct Donate<'info> {
    #[account(
        mut,
       seeds=[OWNER_PREFIX, campaign_name.as_ref()],
        bump,
      )]
    pub campaign: Account<'info, Campaign>,

    #[account(
            mut,
            seeds=[VAULT_PREFIX,mint_of_token_being_sent.key().as_ref(), campaign_name.as_ref()],
            bump
        )]
    vault_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    user: Signer<'info>,

    #[account(mut)]
    user_token_account: Account<'info, TokenAccount>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[account]
pub struct Campaign {
    pub owner: Pubkey,
    pub name: String,
    pub amount_raised: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

