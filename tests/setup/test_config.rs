use litesvm::LiteSVM;
use litesvm_token::{
    get_spl_account, spl_token, spl_token::state::Account as TokenAccount,
    CreateAssociatedTokenAccount, CreateMint, MintTo,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use zaals_finance_client::{CAPITAL_PROGRAM_ID, NFT_PROGRAM_ID};

use crate::setup::{capital_accounts::get_position_vault_pda, constants::DECIMALS, utils};

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
    pub fn new() -> TestConfig {
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

pub struct Tokens {
    pub reward_mint: Pubkey,
    pub lock_mint: Pubkey,
    pub provider_reward_ata: Pubkey,
    pub provider_lock_ata: Pubkey,
    pub agent_reward_ata: Pubkey,
    pub vault_reward_ata: Pubkey,
    pub vault_lock_ata: Pubkey,
    pub collection: Keypair
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
        let provider_reward_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&test_config.capital_provider.pubkey())
                .send()
                .unwrap();
        let provider_lock_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &lock_mint)
                .owner(&test_config.capital_provider.pubkey())
                .send()
                .unwrap();
        let agent_reward_ata =
            CreateAssociatedTokenAccount::new(svm, &test_config.god, &reward_mint)
                .owner(&test_config.agent.pubkey())
                .send()
                .unwrap();
        let position_vault = get_position_vault_pda(test_config.capital_provider.pubkey());
        let vault_reward_ata = get_ata(reward_mint, position_vault);
        let vault_lock_ata = get_ata(lock_mint, position_vault);
        let collection = Keypair::new();
        Tokens {
            reward_mint,
            lock_mint,
            provider_reward_ata,
            provider_lock_ata,
            agent_reward_ata,
            vault_lock_ata,
            vault_reward_ata,
            collection
        }
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
}

pub fn get_ata(mint: Pubkey, owner: Pubkey) -> Pubkey {
    let (ata, bump) = Pubkey::find_program_address(
        &[owner.as_ref(), spl_token::id().as_ref(), mint.as_ref()],
        &spl_token::id(), // ATA program
    );
    ata
}
