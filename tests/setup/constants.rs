use solana_sdk::native_token::LAMPORTS_PER_SOL;

pub const NFT_PROGRAM_KEY_PAIR: &str = "target/deploy/nft_program-keypair.json";
pub const NFT_PROGRAM_SO_FILE: &str = "target/deploy/nft_program.so";
pub const CAPITAL_PROGRAM_KEY_PAIR: &str = "target/deploy/capital_program-keypair.json";
pub const CAPITAL_PROGRAM_SO_FILE: &str = "target/deploy/capital_program.so";

pub const TRANSFER_AMOUNT: u64 = 100 * LAMPORTS_PER_SOL;
pub const DECIMALS:u8 = 9;

pub const LOCKING_AMOUNT:u64 = 1000 * LAMPORTS_PER_SOL;
