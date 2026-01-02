#![allow(dead_code)]

use crate::setup::test_config::TestConfig;
use litesvm::LiteSVM;
use litesvm_token::{
    get_spl_account, spl_token, spl_token::state::Account as TokenAccount, MintTo,
};
use solana_sdk::pubkey::Pubkey;
use zaals_finance_client::{
    capital_program::accounts::{AuthorityConfig, Vault},
    CAPITAL_PROGRAM_ID,
};
use zaals_finance_client::{nft_program::accounts::NFTConfig, NFT_PROGRAM_ID};

pub trait FromAccountBytes: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error>;
}
#[macro_export]
macro_rules! impl_from_account_bytes {
    ($t:ty) => {
        impl FromAccountBytes for $t {
            #[inline(always)]
            fn from_bytes(data: &[u8]) -> Result<Self, std::io::Error> {
                <$t>::from_bytes(data)
            }
        }
    };
}

impl_from_account_bytes!(AuthorityConfig);
impl_from_account_bytes!(Vault);
impl_from_account_bytes!(NFTConfig);

pub fn get_ata(mint: Pubkey, owner: Pubkey) -> Pubkey {
    let (ata, _) = Pubkey::find_program_address(
        &[owner.as_ref(), spl_token::id().as_ref(), mint.as_ref()],
        &spl_token::id(), // ATA program
    );
    ata
}

pub fn get_nft_config_pda() -> Pubkey {
    let try_find_program_address =
        Pubkey::try_find_program_address(&[b"NFT_Config"], &NFT_PROGRAM_ID);
    let config = try_find_program_address;
    config.unwrap().0
}

pub fn get_authority_config_pda() -> Pubkey {
    let authority_config = Pubkey::try_find_program_address(&[b"Config"], &CAPITAL_PROGRAM_ID);
    authority_config.unwrap().0
}

#[allow(dead_code)]
pub fn get_position_vault_pda(provider: Pubkey) -> Pubkey {
    let position_vault =
        Pubkey::try_find_program_address(&[b"Vault", provider.as_ref()], &CAPITAL_PROGRAM_ID);
    position_vault.unwrap().0
}

#[allow(dead_code)]
pub fn get_data_from_pda_address<T>(svm: &mut LiteSVM, pda_address: Pubkey) -> T
where
    T: FromAccountBytes,
{
    let account = svm
        .get_account(&pda_address)
        .expect("Unable to find the PDA {pda_address}");
    T::from_bytes(&account.data).expect("unable to deserialize data")
}

pub fn fund_ata(
    svm: &mut LiteSVM,
    test_config: TestConfig,
    ata: Pubkey,
    mint: Pubkey,
    amount: u64,
) {
    MintTo::new(svm, &test_config.god, &mint, &ata, amount)
        .owner(&test_config.god)
        .send()
        .unwrap()
}

pub fn get_ata_balance(svm: &mut LiteSVM, ata: Pubkey) -> u64 {
    let ata_account: TokenAccount = get_spl_account(svm, &ata).unwrap();
    ata_account.amount
}
