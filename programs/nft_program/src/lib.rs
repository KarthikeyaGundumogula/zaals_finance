#![allow(deprecated)]
use anchor_lang::prelude::*;
declare_id!("AkFAoXys2zhqE15q8XJJJRqXgxLdtJ1kb9ec4fCo1GgH");

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use events::*;
use instructions::*;

#[program]
pub mod nft_program {

    use super::*;

    pub fn init_nft_program_handler(
        ctx: Context<InitNFTProgram>,
        capital_program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps, capital_program)?;
        msg!("Program initialized");
        emit!(ProgramInitializedEvent {
            capital_program,
            admin: *ctx.accounts.admin.key,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn create_vault_collection_handler(ctx: Context<CreateVaultCollection>) -> Result<()> {
        ctx.accounts.create_collection()?;
        msg!("Vault collection created");
        emit!(CollectionCreatedEvent {
            collection: *ctx.accounts.collection.key,
            update_authority: *ctx.accounts.update_authority.key,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn create_core_asset_handler(
        ctx: Context<CreateAsset>,
        args: CreateAssetArgs,
    ) -> Result<()> {
        ctx.accounts.create_asset(args)?;
        msg!("Core asset created");
        emit!(AssetMintedEvent {
            owner: *ctx.accounts.owner.key,
            asset: *ctx.accounts.asset.key,
            collection: *ctx.accounts.collection.key,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn list_asset_handler(
        ctx: Context<ListPosition>,
        price: u64,
        paying_token_mint: Pubkey,
    ) -> Result<()> {
        ctx.accounts
            .create_offer(price, paying_token_mint, ctx.bumps)?;
        ctx.accounts.lock_asset()?;
        msg!("Asset listed for sale");
        emit!(OfferCreatedEvent {
            seller: *ctx.accounts.seller.key,
            price,
            token_mint: paying_token_mint,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn unlist_asset_handler(ctx: Context<UnlistPosition>) -> Result<()> {
        ctx.accounts.unlock_asset()?;
        msg!("Asset unlisted from sale");
        emit!(OfferCancelledEvent {
            seller: *ctx.accounts.seller.key,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
    pub fn burn_asset_handler(ctx: Context<BurnAsset>) -> Result<()> {
        ctx.accounts.burn()?;
        msg!("Asset burned");
        emit!(AssetBurnedEvent {
            owner: *ctx.accounts.holder.key,
            asset: *ctx.accounts.asset.key,
            collection: *ctx.accounts.collection.key,
            time_stamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Vault should be the owner of NFT")]
    InvalidOwner,
}
