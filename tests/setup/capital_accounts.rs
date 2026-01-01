use litesvm::LiteSVM;
use solana_sdk::pubkey::Pubkey;
use zaals_finance_client::{
    capital_program::accounts::{AuthorityConfig, Vault},
    CAPITAL_PROGRAM_ID,
};


#[allow(dead_code)]
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
