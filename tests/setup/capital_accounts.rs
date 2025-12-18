use litesvm::LiteSVM;
use solana_sdk::pubkey::Pubkey;
use zaals_finance_client::{accounts::AuthorityConfig, CAPITAL_PROGRAM_ID};

pub fn get_authority_config_pda() -> Pubkey {
    let authority_config = Pubkey::try_find_program_address(&[b"Config"], &CAPITAL_PROGRAM_ID);
    authority_config.unwrap().0
}

#[allow(dead_code)]
pub fn get_authority_config_pda_data(svm: &mut LiteSVM) -> AuthorityConfig {
    let authority_address = get_authority_config_pda();
    let account = svm
        .get_account(&authority_address)
        .expect("Authority Config account not found");
    AuthorityConfig::from_bytes(&account.data).expect("Unable Deserialize data")
}
