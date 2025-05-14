fhdbdbduse anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111111111111111"); // Temporary

#[program]
pub mod text_poster {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, banned_words: Vec<String>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.owner = *ctx.accounts.owner.key;
        config.banned_words = banned_words;
        Ok(())
    }
    pub fn post_text(ctx: Context<PostText>, text: String) -> Result<()> {
        require!(text.len() <= 280, ErrorCode::TextTooLong);
        for word in text.split_whitespace() {
            require!(!ctx.accounts.config.banned_words.contains(&word.to_string()), ErrorCode::BannedWord);
        }
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, 100 * 10u64.pow(6))?;
        let post = &mut ctx.accounts.post;
        post.user = *ctx.accounts.user.key;
        post.text = text;
        emit!(PostEvent {
            user: *ctx.accounts.user.key,
            text: post.text.clone(),
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + Config::LEN)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PostText<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,
    #[account(init, payer = user, space = 8 + Post::LEN)]
    pub post: Account<'info, Post>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_token_account.mint == token_mint.key())]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Token>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Config {
    pub owner: Pubkey,
    pub banned_words: Vec<String>,
}

#[account]
pub struct Post {
    pub user: Pubkey,
    pub text: String,
}

#[event]
pub struct PostEvent {
    pub user: Pubkey,
    pub text: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Text is too long")]
    TextTooLong,
    #[msg("Contains banned word")]
    BannedWord,
}

impl Config {
    pub const LEN: usize = 32 + 4 + 50 * 32;
}

impl Post {
    pub const LEN: usize = 32 + 4 + 280;
}