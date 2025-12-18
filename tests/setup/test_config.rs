use litesvm::LiteSVM;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use zaals_finance_client::{CAPITAL_PROGRAM_ID, NFT_PROGRAM_ID};

use crate::setup::utils;

#[allow(dead_code)]
pub struct TestConfig {
    pub nft_program_id: Pubkey,
    pub capital_program_id: Pubkey,
    pub svm: LiteSVM,
    pub admin: Keypair,
    pub agent: Keypair,
    pub god: Keypair,
    pub node_operator: Keypair,
    pub capital_provider: Keypair,
    pub slash_claimant: Pubkey,
    pub beneficiary_1: Keypair,
    pub beneficiary_2: Keypair,
    pub beneficiary_3: Keypair,
    pub beneficiary_4: Keypair,
    pub beneficiary_5: Keypair,
    pub extra_beneficiary: Keypair,
}

impl TestConfig {
    pub fn new() -> self::TestConfig {
        let mut svm = LiteSVM::new().with_sysvars();
        let agent = Keypair::new(); //("Agent ATHREYA");
        let admin = Keypair::new(); //("I'm GOD");
        let god = Keypair::new(); //("I'm PAYING GOD");
        let node_operator = Keypair::new(); //("I'M YOUR GAME BD PROVIDER");
        let capital_provider = Keypair::new(); //("I INVEST MY SAVINGS");
        let slash_claimant = Pubkey::new_unique(); //("I LOST SERVICE");
        let beneficiary_1 = Keypair::new(); //("OWNS WAREHOUSE");
        let beneficiary_2 = Keypair::new(); //("OWNS HARDWARE");
        let beneficiary_3 = Keypair::new(); //("SYSTEMS_ENGINEER");
        let beneficiary_4 = Keypair::new(); //("SETUP COSTS LENDER");
        let beneficiary_5 = Keypair::new(); //("SECURITY GAURD");
        let extra_beneficiary = Keypair::new(); //("LATE-COMER");
        let capital_program_id = CAPITAL_PROGRAM_ID;
        let nft_program_id = NFT_PROGRAM_ID;

        utils::fund(&mut svm, agent.pubkey()).expect("airdrop failed");
        utils::fund(&mut svm, admin.pubkey()).expect("airdrop failed");
        utils::fund(&mut svm, node_operator.pubkey()).expect("airdrop failed");
        utils::fund(&mut svm, capital_provider.pubkey()).expect("airdrop failed");
        utils::fund(&mut svm, god.pubkey()).expect("airdrop failed");

        utils::deploy_nft_program(&mut svm).expect("nft_program deployment failed");
        utils::deploy_capital_program(&mut svm).expect("capital_program deployment failed");

        TestConfig {
            nft_program_id,
            capital_program_id,
            svm,
            admin,
            agent,
            god,
            node_operator,
            capital_provider,
            slash_claimant,
            beneficiary_1,
            beneficiary_2,
            beneficiary_3,
            beneficiary_4,
            beneficiary_5,
            extra_beneficiary,
        }
    }
}
