use crate::{errors::*, state::Offer};
use anchor_lang::prelude::*;
use mpl_core::{accounts::BaseAssetV1, instructions::TransferV1CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct ListPosition<'info> {
    #[account(mut,address = asset.owner @SignerError::InvalidAssetOwner)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
        init,
        payer = seller,
        space = Offer::INIT_SPACE + 8,
        seeds = [b"Offer",asset.key().as_ref()],
        bump
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

impl<'info> ListPosition<'info> {
    pub fn create_offer(
        &mut self,
        price: u64,
        paying_token_mint: Pubkey,
        bumps: ListPositionBumps,
    ) -> Result<()> {
        self.offer.set_inner(Offer {
            seller: *self.seller.key,
            price,
            token_mint: paying_token_mint,
            bump: bumps.offer,
        });
        Ok(())
    }

    pub fn lock_asset(&mut self) -> Result<()> {
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.seller.to_account_info()))
            .new_owner(&self.offer.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .payer(&self.seller.to_account_info())
            .invoke()?;
        Ok(())
    }
}
