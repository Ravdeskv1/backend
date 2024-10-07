use anchor_lang::prelude::*;
use anchor_spl_governance::governance_program;

declare_id!("Your_Program_ID_Here");

#[program]
pub mod solana_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, buyer: Pubkey, seller: Pubkey, total_amount: u64, milestone_count: u8) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        
        escrow_account.buyer = buyer;
        escrow_account.seller = seller;
        escrow_account.total_amount = total_amount;
        escrow_account.milestone_count = milestone_count;
        escrow_account.completed_milestones = 0;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let buyer = &ctx.accounts.buyer;
        let system_program = &ctx.accounts.system_program;

        // Transfer funds from buyer to escrow account
        anchor_lang::solana_program::program::invoke(
            &governance_program::transfer_lamports(
                CpiContext::new(system_program.to_account_info(), governance_program::TransferLamports {
                    from_pubkey: buyer.key(),
                    to_pubkey: escrow_account.key(),
                    lamports: amount,
                }),
            ),
            &[],
        )?;

        escrow_account.deposited_amount += amount;

        Ok(())
    }

    pub fn release_milestone(ctx: Context<ReleaseMilestone>, milestone_index: u8) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let seller = &ctx.accounts.seller;
        let system_program = &ctx.accounts.system_program;

        require!(milestone_index < escrow_account.milestone_count, ErrorCode::InvalidMilestoneIndex);
        require!(escrow_account.completed_milestones < milestone_index, ErrorCode::MilestoneAlreadyCompleted);
        
        let amount_to_release = escrow_account.total_amount / escrow_account.milestone_count as u64;

        // Transfer funds from escrow account to seller
        anchor_lang::solana_program::program::invoke(
            &governance_program::transfer_lamports(
                CpiContext::new(system_program.to_account_info(), governance_program::TransferLamports {
                    from_pubkey: escrow_account.key(),
                    to_pubkey: seller.key(),
                    lamports: amount_to_release,
                }),
            ),
            &[],
        )?;

        escrow_account.completed_milestones += 1;

        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let buyer = &ctx.accounts.buyer;
        let seller = &ctx.accounts.seller;
        let system_program = &ctx.accounts.system_program;

        require!(escrow_account.completed_milestones == escrow_account.milestone_count, ErrorCode::NotAllMilestonesCompleted);

        // Transfer any remaining funds to the buyer
        let remaining_amount = escrow_account.deposited_amount - escrow_account.total_amount * escrow_account.completed_milestones as u64;
        if remaining_amount > 0 {
            anchor_lang::solana_program::program::invoke(
                &governance_program::transfer_lamports(
                    CpiContext::new(system_program.to_account_info(), governance_program::TransferLamports {
                        from_pubkey: escrow_account.key(),
                        to_pubkey: buyer.key(),
                        lamports: remaining_amount,
                    }),
                ),
                &[],
            )?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = buyer, space = 8 + 40 + 32 + 8 + 8 + 8)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReleaseMilestone<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct EscrowAccount {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub total_amount: u64,
    pub milestone_count: u8,
    pub completed_milestones: u8,
    pub deposited_amount: u64,
}

#[error]
pub enum ErrorCode {
    #[msg("Invalid milestone index")]
    InvalidMilestoneIndex,
    #[msg("Milestone already completed")]
    MilestoneAlreadyCompleted,
    #[msg("Not all milestones completed")]
    NotAllMilestonesCompleted,
}