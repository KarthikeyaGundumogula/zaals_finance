use solana_sdk::native_token::LAMPORTS_PER_SOL;

pub const NFT_PROGRAM_KEY_PAIR: &str = "target/deploy/nft_program-keypair.json";
pub const NFT_PROGRAM_SO_FILE: &str = "target/deploy/nft_program.so";
pub const CAPITAL_PROGRAM_KEY_PAIR: &str = "target/deploy/capital_program-keypair.json";
pub const CAPITAL_PROGRAM_SO_FILE: &str = "target/deploy/capital_program.so";

pub const MPL_CORE_ID: &str = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
pub const MPL_CORE_SO_FILE: &str = "tests/programs/core.so";
pub const ONE_DAY: i64 = 86_400;
pub const FUND_RAISE_PERIOD: i64 = 10 * 86_400;
pub const MAX_VAULT_THRESHOLD: u64 = 10_000 * LAMPORTS_PER_SOL;
pub const MAX_SLASH_BPS: u16 = 2_000;
pub const INVESTOR_BPS: u16 = 5_000;
pub const MIN_LOCK_AMOUNT: u64 = 1 * LAMPORTS_PER_SOL;
pub const MIN_VAULT_TARGET: u64 = 20 * LAMPORTS_PER_SOL;

pub const TRANSFER_AMOUNT: u64 = 100 * LAMPORTS_PER_SOL;
pub const DECIMALS: u8 = 9;

pub const LOCKING_AMOUNT: u64 = 1000 * LAMPORTS_PER_SOL;
