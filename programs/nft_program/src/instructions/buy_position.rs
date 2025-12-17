use crate::{errors::*, state::Offer};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::{accounts::BaseAssetV1, instructions::TransferV1CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct BuyPosition<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: seller must match the offer lister
    #[account(
        address = offer.seller @ OfferError::InvalidSeller
    )]
    pub seller: UncheckedAccount<'info>,
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
      mut,
      close = buyer,
      seeds = [b"Offer",asset.key().as_ref()],
      bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,
    /// The collection to which the asset belongs.
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub collection: AccountInfo<'info>,
    /// Reward token mint
    #[account(
        mint::token_program = token_program,
        address = offer.token_mint @ OfferError::InvalidMint
    )]
    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = seller,
        associated_token::token_program = token_program
    )]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: this will be checked my mpl-core-program
    #[account(
        address = MPL_CORE_ID @ ExteranlProgramError::InvalidMPLCoreProgramId
    )]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyPosition<'info> {
    pub fn transfer_tokens(&mut self) -> Result<()> {
        let transfer_accounts = TransferChecked {
            from: self.buyer_ata.to_account_info(),
            to: self.seller_ata.to_account_info(),
            mint: self.token_mint.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);
        transfer_checked(cpi_ctx, self.offer.price, self.token_mint.decimals)?;
        Ok(())
    }
    pub fn transfer_asset(&mut self) -> Result<()> {
        let offer_seed = b"Offer";
        let asset_key = self.asset.key();
        let bump_seed = [self.offer.bump];
        let seeds = &[offer_seed.as_ref(), asset_key.as_ref(), bump_seed.as_ref()];
        let signers = &[&seeds[..]];
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.offer.to_account_info()))
            .new_owner(&self.buyer.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .payer(&self.buyer.to_account_info())
            .invoke_signed(signers)?;
        Ok(())
    }
}
