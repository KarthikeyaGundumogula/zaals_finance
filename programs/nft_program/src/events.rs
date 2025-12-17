use anchor_lang::prelude::*;

#[event]
pub struct ProgramInitializedEvent {
    pub capital_program: Pubkey,
    pub admin: Pubkey,
    pub time_stamp: i64,
}

#[event]
pub struct CollectionCreatedEvent {
    pub collection: Pubkey,
    pub update_authority: Pubkey,
    pub time_stamp: i64,
}

#[event]
pub struct AssetMintedEvent {
    pub owner: Pubkey,
    pub asset: Pubkey,
    pub collection: Pubkey,
    pub time_stamp: i64,
}

#[event]
pub struct AssetBurnedEvent {
    pub owner: Pubkey,
    pub asset: Pubkey,
    pub collection: Pubkey,
    pub time_stamp: i64,
}

#[event]
pub struct OfferCreatedEvent {
    pub seller: Pubkey,
    pub price: u64,
    pub token_mint: Pubkey,
    pub time_stamp: i64,
}

#[event]
pub struct OfferCancelledEvent {
    pub seller: Pubkey,
    pub time_stamp: i64,
}
#[event]
pub struct OfferPurchasedEvent {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub price: u64,
    pub token_mint: Pubkey,
    pub time_stamp: i64,
}
