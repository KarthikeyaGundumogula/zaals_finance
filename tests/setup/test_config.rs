#![allow(unused)]
use anchor_spl::associated_token::get_associated_token_address;
use litesvm::LiteSVM;
use litesvm_token::{
    get_spl_account, spl_token, spl_token::state::Account as TokenAccount,
    CreateAssociatedTokenAccount, CreateMint,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use zaals_finance_client::{
    capital_program::types::Beneficiary, CAPITAL_PROGRAM_ID, NFT_PROGRAM_ID,
};

use crate::setup::{accounts, constants::DECIMALS, utils};

#[allow(dead_code)]
pub struct TestConfig {
    pub svm: LiteSVM,
    pub admin: Keypair,
    pub agent: Keypair,
    pub god: Keypair,
    pub node_operator: Keypair,
    pub capital_provider: Keypair,
    pub slash_claimant: Pubkey,
    pub beneficiary_signers: Vec<Keypair>,
    pub beneficiaries: Vec<Beneficiary>,
    pub extra_beneficiary: Keypair,
}

impl TestConfig {
    pub fn new() -> TestConfig {
        let mut svm = LiteSVM::new().with_sysvars();
        let agent = Keypair::new(); //("Agent ATHREYA");
        let admin = Keypair::new(); //("I'm GOD");
        let god = Keypair::new(); //("I'm PAYING GOD");
        let node_operator = Keypair::new(); //("I'M YOUR GAME BD PROVIDER");
        let capital_provider = Keypair::new(); //("I INVEST MY SAVINGS");
        let slash_claimant = Pubkey::new_unique(); //("I LOST SERVICE");
                                                   // beneficiary_1 -- OWNS WAREHOUSE
                                                   // beneficiary_2 -- OWNS HARDWARE
                                                   // beneficiary_3 -- SYSTEMS_ENGINEER
                                                   // beneficiary_4 -- SETUP COSTS LENDER
                                                   // beneficiary_5 -- SECURITY GAURD

        let mut beneficiaries: Vec<Beneficiary> = Vec::new();
        let mut beneficiary_signers: Vec<Keypair> = Vec::new();
        for i in 1..6 {
            let b = Keypair::new();
            beneficiaries.push(Beneficiary {
                address: b.pubkey(),
                share_bps: i * 250,
                total_claimed: 0,
            });
            beneficiary_signers.push(b);
        }
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
        utils::deploy_mpl_program(&mut svm).expect("mpl_core deployment failed");

        TestConfig {
            svm,
            admin,
            agent,
            god,
            beneficiaries,
            beneficiary_signers,
            node_operator,
            capital_provider,
            slash_claimant,
            extra_beneficiary,
        }
    }
}

pub struct Tokens {
    pub reward_mint: Pubkey,
    pub lock_mint: Pubkey,
    pub node_operator_reward_ata: Pubkey,
    pub agent_reward_ata: Pubkey,
    pub provider_lock_ata: Pubkey,
    pub provider_reward_ata: Pubkey,
    pub vault_reward_ata: Pubkey,
    pub vault_lock_ata: Pubkey,
    pub admin_lock_ata: Pubkey,
    pub beneficiary_atas: Vec<Pubkey>,
    pub collection: Keypair,
    pub asset: Keypair,
}

impl Tokens {
    pub fn create(test_config: &mut TestConfig) -> Tokens {
        let svm = &mut test_config.svm;
        // Create a new SPL token mint with alice as the mint authority
        let reward_mint = CreateMint::new(svm, &test_config.god)
            .authority(&test_config.god.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();
        let lock_mint = CreateMint::new(svm, &test_config.god)
            .authority(&test_config.god.pubkey())
            .decimals(DECIMALS)
            .send()
            .unwrap();
        let provider_lock_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &lock_mint)
                .owner(&test_config.capital_provider.pubkey())
                .send()
                .unwrap();
        let admin_lock_ata = CreateAssociatedTokenAccount::new(svm, &test_config.god, &lock_mint)
            .owner(&test_config.admin.pubkey())
            .send()
            .unwrap();
        let agent_reward_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&test_config.agent.pubkey())
                .send()
                .unwrap();
        let node_operator_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&test_config.node_operator.pubkey())
                .send()
                .unwrap();
        let position_vault = accounts::get_vault_pda(test_config.node_operator.pubkey());
        let vault_lock_ata = CreateAssociatedTokenAccount::new(svm, &test_config.god, &lock_mint)
            .owner(&position_vault)
            .send()
            .unwrap();
        let vault_reward_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&position_vault)
                .send()
                .unwrap();
        let provider_reward_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&test_config.capital_provider.pubkey())
                .send()
                .unwrap();
        let mut beneficiary_atas: Vec<Pubkey> = Vec::new();
        for beneficiary in &test_config.beneficiaries {
            let ata = CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&beneficiary.address)
                .send()
                .unwrap();
            beneficiary_atas.push(ata);
        }
        let collection = Keypair::new();
        let asset = Keypair::new();
        Tokens {
            reward_mint,
            lock_mint,
            node_operator_reward_ata: node_operator_ata,
            agent_reward_ata,
            provider_lock_ata,
            admin_lock_ata,
            provider_reward_ata,
            vault_lock_ata,
            vault_reward_ata,
            beneficiary_atas,
            collection,
            asset,
        }
    }
}
