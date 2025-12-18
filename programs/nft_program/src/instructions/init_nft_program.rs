use crate::state::NFTConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitNFTProgram<'info> {
    #[account(
    init,
    seeds = [b"NFT_Config"],
    bump,
    payer = admin,
    space = NFTConfig::INIT_SPACE + 8
  )]
    pub config: Account<'info, NFTConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitNFTProgram<'info> {
    pub fn initialize(
        &mut self,
        bumps: InitNFTProgramBumps,
        capital_program: Pubkey,
    ) -> Result<()> {
        self.config.set_inner(NFTConfig {
            capital_program: capital_program,
            authority: *self.authority.key,
            admin: *self.admin.key,
            bump: bumps.config,
        });
        Ok(())
    }
}
