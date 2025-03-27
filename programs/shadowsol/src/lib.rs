use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Transfer};
use std::mem::size_of;

pub mod groth16_solana;
mod verifying_key;
mod verifier;
mod merkle;

use verifier::verify_proof_logic;
use merkle::is_known_root;
// ID программы
declare_id!("9yQAM7m1Vc8C2uDPn3EgEtMaG3wWPWetS9PiyRJtyNuV");

pub const ROOT_HISTORY_SIZE: u32 = 16;
pub const LEVELS: u32 = 2;

/// Запись для маппинга nullifierHashes (каждая запись — значение типа bool)
#[account]
pub struct NullifierEntry {
    pub value: bool,
}

/// Запись для маппинга commitments (аналогично)
#[account]
pub struct CommitmentEntry {
    pub value: bool,
}

/// Глобальное состояние ShadowSol
#[account]
pub struct ShadowState {
    pub denomination: u64,        
    pub token_mint: Pubkey,       
    pub levels: u32,              
    pub filled_subtrees: [[u8; 32]; LEVELS as usize],
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE as usize],
    pub current_root_index: u32,
    pub next_index: u32,
}

#[program]
pub mod shadow_sol {
    use super::*;

    pub fn initialize_global(
        ctx: Context<InitializeGlobal>,
        denomination: u64,
        token_mint: Pubkey
    ) -> Result<()> {
        let state = &mut ctx.accounts.shadow_state;
        state.denomination = denomination;
        state.token_mint = token_mint;
        state.levels = LEVELS;
        state.filled_subtrees = std::array::from_fn(|i| merkle::zeros(i as u32));

        let mut roots = [[0u8; 32]; ROOT_HISTORY_SIZE as usize];
        roots[0] = merkle::zeros(LEVELS - 1);
        state.roots = roots;
        state.current_root_index = 0;
        state.next_index = 0;
        Ok(())
    }

    pub fn deposit(
        ctx: Context<DepositCtx>,
        commitment: [u8; 32],
    ) -> Result<()> {
        let state = &mut ctx.accounts.shadow_state;

        require!(
            !ctx.accounts.commitment_entry.value,
            CustomError::CommitmentAlreadySubmitted
        );

        merkle::insert_leaf(state, commitment)?;

        ctx.accounts.commitment_entry.value = true;

        // Переводим `state.denomination` токенов
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.program_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            state.denomination,
        )?;

        Ok(())
    }

    pub fn withdraw(
        ctx: Context<WithdrawCtx>,
        proof: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: [u8; 32],
        token: [u8; 32],
    ) -> Result<()> {
        let state = &mut ctx.accounts.shadow_state;

        require!(
            !ctx.accounts.nullifier_entry.value,
            CustomError::NoteAlreadySpent
        );

        require!(
            is_known_root(state, root),
            CustomError::UnknownMerkleRoot
        );

        let mut public_inputs = Vec::new();
        public_inputs.extend_from_slice(&root);
        public_inputs.extend_from_slice(&nullifier_hash);
        public_inputs.extend_from_slice(&recipient);
        public_inputs.extend_from_slice(&token);

        verifier::verify_proof_logic(proof, public_inputs)?;

        msg!("Proof is valid. Performing withdraw logic...");

        ctx.accounts.nullifier_entry.value = true;

        // Переводим `state.denomination` токенов обратно пользователю
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.program_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.program_signer.to_account_info(),
                },
            ),
            state.denomination,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 8 + 32 + 4 + (LEVELS as usize * 32) + (ROOT_HISTORY_SIZE as usize * 32) + 4 + 4,
    )]
    pub shadow_state: Account<'info, ShadowState>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(commitment: [u8; 32])]
pub struct DepositCtx<'info> {
    #[account(mut)]
    pub shadow_state: Account<'info, ShadowState>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + 1,
        seeds = [b"commitment", &commitment[..]],
        bump
    )]
    pub commitment_entry: Account<'info, CommitmentEntry>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub system_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(proof: Vec<u8>, root: [u8; 32], nullifier_hash: [u8; 32], recipient: [u8; 32], token: [u8; 32])]
pub struct WithdrawCtx<'info> {
    #[account(mut)]
    pub shadow_state: Account<'info, ShadowState>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 1,
        seeds = [b"nullifier", &nullifier_hash[..]],
        bump
    )]
    pub nullifier_entry: Account<'info, NullifierEntry>,

    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub program_signer: Signer<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub system_program: UncheckedAccount<'info>,
}

#[error_code]
pub enum CustomError {
    #[msg("Commitment already submitted")]
    CommitmentAlreadySubmitted,
    #[msg("Note has already been spent")]
    NoteAlreadySpent,
    #[msg("Unknown Merkle root")]
    UnknownMerkleRoot,
    #[msg("Commitment insertion error")]
    InsertionError,
}
