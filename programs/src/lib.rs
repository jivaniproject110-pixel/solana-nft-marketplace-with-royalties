```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Transfer};
use metaplex_program::state::Royalty;
use metaplex_program::instructions::create_metadata_account_v3;
use solana_program::program::invoke;
use spl_token::instruction::approve;
use std::convert::TryInto;

declare_id!("YourProgramID");

#[program]
pub mod solana_nft_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, config: InitializeConfig) -> Result<()> {
        msg!("Initializing marketplace...");

        // Initialize marketplace account
        let market_account = &mut ctx.accounts.market_account;
        market_account.authority = *ctx.accounts.authority.key;
        market_account.version = 1;
        market_account.name = config.name.to_string();
        market_account.symbol = config.symbol.to_string();
        market_account.mint = ctx.accounts.mint.key();
        market_account.decimals = config.decimals;

        // Set authority of mint account
        invoke(
            SetAuthority {
                account_or_mint: ctx.accounts.mint.key(),
                current_authority: ctx.accounts.authority.key(),
                authority_type: AuthorityType::MintTo,
                is_constraint_exempt: true,
            },
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.mint.clone(),
            ],
        )?;

        // Create and initialize NFT token account
        let (nft_token_account, bump) = Pubkey::find_program_address(
            &[
                b"nft-token-account".as_ref(),
                ctx.accounts.market_account.key().as_ref(),
            ],
            ctx.program_id,
        );
        ctx.accounts.nft_token_account = nft_token_account;

        // Initialize NFT metadata account
        create_metadata_account_v3(
            ctx.accounts.authority.key,
            ctx.accounts.nft_token_account,
            ctx.accounts.market_account.mint,
            ctx.accounts.authority.key,
        )
    }

    pub fn create_nft(ctx: Context<CreateNft>, config: CreateNftConfig) -> Result<()> {
        msg!("Creating NFT...");

        // Initialize NFT account
        let nft_account = &mut ctx.accounts.nft_account;
        nft_account.authority = *ctx.accounts.authority.key;
        nft_account.version = 1;
        nft_account.mint = ctx.accounts.market_account.mint;
        nft_account.name = config.name.to_string();
        nft_account.symbol = config.symbol.to_string();
        nft_account.uri = config.uri.to_string();

        // Set authority of NFT account
        invoke(
            SetAuthority {
                account_or_mint: ctx.accounts.nft_account.key(),
                current_authority: ctx.accounts.authority.key(),
                authority_type: AuthorityType::AccountOwner,
                is_constraint_exempt: true,
            },
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.nft_account.clone(),
            ],
        )?;

        // Approve market account to transfer NFT
        approve(
            ctx.accounts.nft_account.key,
            ctx.accounts.market_account.key,
            ctx.accounts.nft_account.key,
            [0],
            0,
        )?;

        // Create and initialize royalty account
        let (royalty_account, bump) = Pubkey::find_program_address(
            &[
                b"royalty-account".as_ref(),
                ctx.accounts.nft_account.key().as_ref(),
            ],
            ctx.program_id,
        );
        ctx.accounts.royalty_account = royalty_account;

        // Initialize royalty account with creator's address
        let royalty = &mut ctx.accounts.royalty;
        royalty.royalty = config.royalty;
        royalty.owner = ctx.accounts.authority.key;
        royalty.nft_account = ctx.accounts.nft_account.key;

        // Set authority of royalty account
        invoke(
            SetAuthority {
                account_or_mint: ctx.accounts.royalty_account.key(),
                current_authority: ctx.accounts.authority.key(),
                authority_type: AuthorityType::AccountOwner,
                is_constraint_exempt: true,
            },
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.royalty_account.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn auction_nft(ctx: Context<AuctionNft>, config: AuctionNftConfig) -> Result<()> {
        msg!("Auctioning NFT...");

        // Initialize auction account
        let auction_account = &mut ctx.accounts.auction_account;
        auction_account.authority = *ctx.accounts.authority.key;
        auction_account.version = 1;
        auction_account.nft_account = ctx.accounts.nft_account.key;
        auction_account.start_time = config.start_time;
        auction_account.end_time = config.end_time;
        auction_account.reserve_price = config.reserve_price;
        auction_account.highest_bidder = ctx.accounts.authority.key;
        auction_account.highest_bid = 0;

        // Transfer NFT to auction account
        invoke(
            Transfer {
                from: ctx.accounts.nft_account.key,
                to: ctx.accounts.auction_account.key,
                lamports: ctx.accounts.nft_account lamports,
                owner: ctx.accounts.authority.key,
            },
            &[
                ctx.accounts.nft_account.clone(),
                ctx.accounts.auction_account.clone(),
                ctx.accounts.authority.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn bid_on_nft(ctx: Context<BidOnNft>, config: BidOnNftConfig) -> Result<()> {
        msg!("Placing bid on NFT...");

        // Check if auction is still active
        if ctx.accounts.auction_account.end_time <= ctx.accounts.block_info.block_time {
            return Err(Error::Custom(BidOnNftError::AuctionExpired));
        }

        // Update highest bidder and bid amount
        let auction_account = &mut ctx.accounts.auction_account;
        if config.amount > auction_account.highest_bid {
            auction_account.highest_bidder = ctx.accounts.authority.key;
            auction_account.highest_bid = config.amount;
        }

        Ok(())
    }

    pub fn conclude_auction(ctx: Context<ConcludeAuction>) -> Result<()> {
        msg!("Concluding auction...");

        // Check if auction is still active
        if ctx.accounts.auction_account.end_time <= ctx.accounts.block_info.block_time {
            return Err(Error::Custom(ConcludeAuctionError::AuctionExpired));
        }

        // Transfer NFT to highest bidder
        invoke(
            Transfer {
                from: ctx.accounts.auction_account.key,
                to: ctx.accounts.highest_bidder.key,
                lamports: ctx.accounts.auction_account lamports,
                owner: ctx.accounts.authority.key,
            },
            &[
                ctx.accounts.auction_account.clone(),
                ctx.accounts.highest_bidder.clone(),
                ctx.accounts.authority.clone(),
            ],
        )?;

        Ok(())
    }
}

#[error_code]
pub enum Error {
    #[msg("Auction has expired")]
    AuctionExpired,
}

#[error_code]
pub enum BidOnNftError {
    #[msg("Auction has expired")]
    AuctionExpired,
}

#[error_code]
pub enum ConcludeAuctionError {
    #[msg("Auction has expired")]
    AuctionExpired,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 300)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(address = spl_token::token::ID)]
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
pub struct CreateNft<'info> {
    #[account(mut)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(init, payer = authority, space = 300)]
    pub nft_account: Account<'info, NFTAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub royalty_program: Program<'info, metaplex_program::Royalty>,
}

#[derive(Accounts)]
pub struct AuctionNft<'info> {
    #[account(mut)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(init, payer = authority, space = 300)]
    pub auction_account: Account<'info, AuctionAccount>,
    #[account(mut)]
    pub nft_account: Account<'info, NFTAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
pub struct BidOnNft<'info> {
    #[account(mut)]
    pub auction_account: Account<'info, AuctionAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConcludeAuction<'info> {
    #[account(mut)]
    pub auction_account: Account<'info, AuctionAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateNftConfig {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub royalty: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AuctionNftConfig {
    pub start_time: u64,
    pub end_time: u64,
    pub reserve_price: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BidOnNftConfig {
    pub amount: u64,
}

#[derive(Accounts)]
#[accounts(Initialize)]
pub struct MarketAccount {
    pub authority: Pubkey,
    pub version: u8,
    pub name: String,
    pub symbol: String,
    pub mint: Pubkey,
    pub decimals: u8,
}

#[derive(Accounts)]
#[accounts(CreateNft)]
pub struct NFTAccount {
    pub authority: Pubkey,
    pub version: u8,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
#[accounts(AuctionNft)]
pub struct AuctionAccount {
    pub authority: Pubkey,
    pub version: u8,
    pub nft_account: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub reserve_price: u64,
    pub highest_bidder: Pubkey,
    pub highest_bid: u64,
}

#[derive(Accounts)]
#[accounts(ConcludeAuction)]
pub struct HighestBidder {
    pub authority: Pubkey,
}
```

Note that you will need to replace `YourProgramID` with the actual ID of your Solana program. Additionally, this code is a basic implementation and may not cover all edge cases. You should thoroughly test and review the code before deploying it to the Solana network.