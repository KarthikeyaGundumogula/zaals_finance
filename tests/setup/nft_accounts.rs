use litesvm::LiteSVM;
use solana_sdk::pubkey::Pubkey;
use zaals_finance_client::{accounts::NFTConfig, NFT_PROGRAM_ID};

pub fn get_nft_config_pda() -> Pubkey {
    let try_find_program_address = Pubkey::try_find_program_address(&[b"NFT_Config"], &NFT_PROGRAM_ID);
    let config = try_find_program_address;
    config.unwrap().0
}

#[allow(dead_code)]
pub fn get_nft_config_pda_data(svm: &mut LiteSVM) -> NFTConfig {
    let config_address = get_nft_config_pda();
    let account = svm
        .get_account(&config_address)
        .expect("Config account not found");

    NFTConfig::from_bytes(&account.data).expect("Nft Config not found")
}
