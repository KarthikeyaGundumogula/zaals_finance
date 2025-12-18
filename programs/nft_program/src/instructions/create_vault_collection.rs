use anchor_lang::prelude::*;
use mpl_core::{instructions::CreateCollectionV1CpiBuilder, ID as MPL_CORE_ID};

use crate::state::NFTConfig;

#[derive(Accounts)]
pub struct CreateVaultCollection<'info> {
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: performed some check here
    #[account(address = config.authority)]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
       seeds = [b"NFT_Config"],
       bump = config.bump
    )]
    pub config: Account<'info, NFTConfig>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this will be checked by the mpl-core program
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateVaultCollection<'info> {
    pub fn create_collection(&mut self) -> Result<()> {
        CreateCollectionV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .update_authority(Some(self.update_authority.as_ref()))
            .system_program(&self.system_program.to_account_info())
            .payer(&self.payer.to_account_info())
            .name("Capital PositionV1".to_string())
            .uri("H".to_string())
            .invoke()?;
        Ok(())
    }
}
