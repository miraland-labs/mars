use solana_program::{keccak::Hash, pubkey, pubkey::Pubkey};

/// The unix timestamp after which mining can begin.
// Monday, April 23, 2024 2:00:00 PM GMT
pub const START_AT: i64 = 1713880800;

/// The unix timestamp after which mining will stop.
// Sunday, April 23, 2029 2:00:00 PM GMT
pub const END_AT: i64 = 1871647200;

/// The reward rate to intialize the program with.
pub const INITIAL_REWARD_RATE: u64 = 10 * 10u64.pow(3u32);

/// The mining difficulty to initialize the program with.
pub const INITIAL_DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

/// The decimal precision of the MARS token.
/// The smallest indivisible unit of MARS is a miraMARS.
/// 1 miraMARS = 0.00000001 MARS
pub const TOKEN_DECIMALS: u8 = 8;

/// One MARS token, denominated in units of miraMARS.
pub const ONE_MARS: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// Ten MARS token
pub const TEN_MARS: u64 = ONE_MARS.saturating_mul(10);

/// The duration of an epoch, in units of seconds.
pub const EPOCH_DURATION: i64 = 60;

/// The target quantity of MARS to be mined per epoch, in units of miraMARS.
/// Inflation rate ≈ 10 MARS / epoch (min 0, max 20)
pub const TARGET_EPOCH_REWARDS: u64 = TEN_MARS;

/// The maximum quantity of MARS that can be mined per epoch, in units of miraMARS.
pub const MAX_EPOCH_REWARDS: u64 = TEN_MARS.saturating_mul(2);

/// The quantity of MARS each bus is allowed to issue per epoch.
pub const BUS_EPOCH_REWARDS: u64 = MAX_EPOCH_REWARDS.saturating_div(BUS_COUNT as u64);

/// The number of bus accounts, for parallelizing mine operations.
pub const BUS_COUNT: usize = 8;

/// The smoothing factor for reward rate changes. The reward rate cannot change by more or less
/// than a factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 2;

// Assert MAX_EPOCH_REWARDS is evenly divisible by BUS_COUNT.
static_assertions::const_assert!(
    (MAX_EPOCH_REWARDS / BUS_COUNT as u64) * BUS_COUNT as u64 == MAX_EPOCH_REWARDS
);

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the metadata account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the mint account PDA.
pub const MINT: &[u8] = b"mint";

/// The seed of proof account PDAs.
pub const PROOF: &[u8] = b"proof";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// The name for token metadata.
pub const METADATA_NAME: &str = "Mars";

/// The ticker symbol for token metadata.
pub const METADATA_SYMBOL: &str = "MARS";

/// The uri for token metdata.
pub const METADATA_URI: &str = "https://github.com/miraland-labs/resources/blob/main/metadata/mars.json";

/// Noise for deriving the mint PDA.
// ExtremeWayOnMars
pub const MINT_NOISE: [u8; 16] = [
    105, 170, 164, 162, 145, 155, 145, 127, 141, 171, 117, 156, 115, 141, 162, 163,
];

/// The addresses of the bus accounts.
pub const BUS_ADDRESSES: [Pubkey; BUS_COUNT] = [
    pubkey!("6ZcZsUKTv19iFXSnpqjPXzE7joCsEeNpTC5oArFPXzQG"),
    pubkey!("5jNbJehucBoCov1RkyHJQfMNrjSFG7LYZF79hvjDp4u5"),
    pubkey!("Ctsj8HbZFe8oqbx3GRfxgz9g1RyTKHeh92UcsabUK6so"),
    pubkey!("HKo6bDCzHyfYtBtbYeDXs1x3yvDzBWU5EpxhjBWxJB97"),
    pubkey!("82BDDgQnQeFLjyP1mSpPHRPJdEGkiiZtBqEvjFxYL44W"),
    pubkey!("CqacmEv1yENenKrZBUPV8LQZrje8ZiBdr8Y3MsEaS3QQ"),
    pubkey!("Gw4ZkcJkiiLWbqR1qnu2YHg4TsfaEbgUE2Ky721cY1Rv"),
    pubkey!("DkwXmEda41smeoQiztKopXMQjMW5QYLggddqLUz1Hpck"),
];

/// The address of the mint metadata account.
pub const METADATA_ADDRESS: Pubkey = pubkey!("CC8Awvao6Ls5VjREyH78DGPh3SmdjdDZJxTmfsqZmsbJ");

/// The address of the mint account.
pub const MINT_ADDRESS: Pubkey = pubkey!("7RAV5UPRTzxn46kLeA8MiJsdNy9VKc5fip8FWEgTpTHh");

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey = pubkey!("Dk13Cdjnjz2pxbsXbvzJiA2bUSMdsHU7Vf2G8yRGQvwC");
