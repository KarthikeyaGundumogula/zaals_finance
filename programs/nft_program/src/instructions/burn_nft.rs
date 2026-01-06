use anchor_lang::prelude::*;
use mpl_core::{instructions::BurnV1CpiBuilder, ID as MPL_CORE_ID};

#[derive(Accounts)]
pub struct BurnAsset<'info> {
    #[account(mut)]
    pub holder: Signer<'info>,
    /// CHECK: this will be checked by mpl-core-program
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: this will be checked by mpl-core-program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: this will be checked by mpl-core-program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> BurnAsset<'info> {
    pub fn burn(&mut self) -> Result<()> {
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .payer(&self.holder.to_account_info())
            .authority(Some(&self.holder.to_account_info()))
            .collection(Some(&self.collection.as_ref()))
            .invoke()?;
        Ok(())
    }
}
