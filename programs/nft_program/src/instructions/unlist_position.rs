use crate::{errors::*, state::Offer};
use anchor_lang::prelude::*;
use mpl_core::{accounts::BaseAssetV1, instructions::TransferV1CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct UnlistPosition<'info> {
    #[account(mut,address = offer.seller @SignerError::InvalidAssetOwner)]
    pub seller: Signer<'info>,
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
      mut,
      close = seller,
      seeds = [b"Offer",asset.key().as_ref()],
      bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,
    /// The collection to which the asset belongs.
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub collection: AccountInfo<'info>,
    /// CHECK: this will be checked my mpl-core-program
    #[account(
        address = MPL_CORE_ID @ ExteranlProgramError::InvalidMPLCoreProgramId
    )]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnlistPosition<'info> {
    pub fn unlock_asset(&mut self) -> Result<()> {
        let offer_seed = b"Offer";
        let asset_key = self.asset.key();
        let bump_seed = [self.offer.bump];
        let seeds = &[offer_seed.as_ref(), asset_key.as_ref(), bump_seed.as_ref()];
        let signers = &[&seeds[..]];
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.offer.to_account_info()))
            .new_owner(&self.seller.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .payer(&self.seller.to_account_info())
            .invoke_signed(signers)?;
        Ok(())
    }
}
