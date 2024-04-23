#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::str::FromStr;

use mars::{
    instruction::MarsInstruction,
    state::{Bus, Treasury},
    utils::{AccountDeserialize, Discriminator},
    BUS, BUS_ADDRESSES, BUS_COUNT, BUS_EPOCH_REWARDS, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE,
    MAX_EPOCH_REWARDS, MINT_ADDRESS, START_AT, TOKEN_DECIMALS, TREASURY, TREASURY_ADDRESS,
};
use rand::seq::SliceRandom;
use solana_program::{
    clock::Clock,
    epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{AccountState, Mint};

#[tokio::test]
async fn test_reset() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Pdas
    let bus_pdas = vec![
        Pubkey::find_program_address(&[BUS, &[0]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[5]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &mars::id()),
    ];
    // let mint_pda = Pubkey::find_program_address(&[MINT], &mars::id());
    let treasury_tokens_address = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );

    // Submit tx
    let ix = mars::instruction::reset(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Test bus state
    for i in 0..BUS_COUNT {
        let bus_account = banks.get_account(bus_pdas[i].0).await.unwrap().unwrap();
        assert_eq!(bus_account.owner, mars::id());
        let bus = Bus::try_from_bytes(&bus_account.data).unwrap();
        assert_eq!(bus.id as u8, i as u8);
        assert_eq!(bus.rewards, BUS_EPOCH_REWARDS);
    }

    // Test treasury state
    let treasury_account = banks.get_account(TREASURY_ADDRESS).await.unwrap().unwrap();
    assert_eq!(treasury_account.owner, mars::id());
    let treasury = Treasury::try_from_bytes(&treasury_account.data).unwrap();
    assert_eq!(
        treasury.admin,
        Pubkey::from_str("staryJacbXodPh4WfwVtgA5jkJhvsMHERtkdttnLEHc").unwrap()
    );
    assert_eq!(treasury.difficulty, INITIAL_DIFFICULTY.into());
    assert_eq!(treasury.last_reset_at, START_AT + 1);
    assert_eq!(treasury.reward_rate, INITIAL_REWARD_RATE.saturating_div(2));
    assert_eq!(treasury.total_claimed_rewards as u8, 0);

    // Test mint state
    let mint_account = banks.get_account(MINT_ADDRESS).await.unwrap().unwrap();
    assert_eq!(mint_account.owner, spl_token::id());
    let mint = Mint::unpack(&mint_account.data).unwrap();
    assert_eq!(mint.mint_authority, COption::Some(TREASURY_ADDRESS));
    assert_eq!(mint.supply, MAX_EPOCH_REWARDS);
    assert_eq!(mint.decimals, mars::TOKEN_DECIMALS);
    assert_eq!(mint.is_initialized, true);
    assert_eq!(mint.freeze_authority, COption::None);

    // Test treasury token state
    let treasury_tokens_account = banks
        .get_account(treasury_tokens_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(treasury_tokens_account.owner, spl_token::id());
    let treasury_tokens = spl_token::state::Account::unpack(&treasury_tokens_account.data).unwrap();
    assert_eq!(treasury_tokens.mint, MINT_ADDRESS);
    assert_eq!(treasury_tokens.owner, TREASURY_ADDRESS);
    assert_eq!(treasury_tokens.amount, MAX_EPOCH_REWARDS);
    assert_eq!(treasury_tokens.delegate, COption::None);
    assert_eq!(treasury_tokens.state, AccountState::Initialized);
    assert_eq!(treasury_tokens.is_native, COption::None);
    assert_eq!(treasury_tokens.delegated_amount, 0);
    assert_eq!(treasury_tokens.close_authority, COption::None);
}

#[tokio::test]
async fn test_reset_bad_key() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Bad addresses
    let bad_pda = Pubkey::find_program_address(&[b"t"], &mars::id());
    for i in 1..13 {
        let mut ix = mars::instruction::reset(payer.pubkey());
        ix.accounts[i].pubkey = bad_pda.0;
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
        let res = banks.process_transaction(tx).await;
        assert!(res.is_err());
    }
}

#[tokio::test]
async fn test_reset_busses_out_of_order_fail() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Pdas
    let signer = payer.pubkey();
    let bus_pdas = vec![
        Pubkey::find_program_address(&[BUS, &[5]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[0]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &mars::id()),
    ];
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );

    // Submit tx
    let ix = Instruction {
        program_id: mars::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MarsInstruction::Reset.to_vec(),
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_reset_race() {
    // Setup
    let (mut banks, payer, payer_alt, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Reset one passes
    let ix = mars::instruction::reset(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_ok());

    // Reset two fails
    let ix = mars::instruction::reset(payer_alt.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer_alt.pubkey()),
        &[&payer_alt],
        blockhash,
    );
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_reset_too_early() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::TooEarly).await;

    // Reset one passes
    let ix = mars::instruction::reset(payer.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_reset_not_enough_keys() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Reset with missing account
    let mut ix = mars::instruction::reset(payer.pubkey());
    ix.accounts.remove(1);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_reset_busses_duplicate_fail() {
    // Setup
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Pdas
    let signer = payer.pubkey();
    let bus_pda = Pubkey::find_program_address(&[BUS, &[0]], &mars::id());
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );

    // Submit tx
    let ix = Instruction {
        program_id: mars::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(bus_pda.0, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: MarsInstruction::Reset.to_vec(),
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
    let res = banks.process_transaction(tx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_reset_shuffle_error() {
    // Setup
    const FUZZ: u64 = 100;
    let (mut banks, payer, _, blockhash) = setup_program_test_env(ClockState::Normal).await;

    // Pdas
    let signer = payer.pubkey();
    let bus_pdas = vec![
        Pubkey::find_program_address(&[BUS, &[5]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[0]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &mars::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &mars::id()),
    ];
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );

    // Fuzz test shuffled accounts.
    // Note some shuffles may still be valid if signer and non-bus accounts are all in correct positions.
    let mut rng = rand::thread_rng();
    for _ in 0..FUZZ {
        let mut accounts = vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];
        accounts.shuffle(&mut rng);
        let ix = Instruction {
            program_id: mars::id(),
            accounts,
            data: MarsInstruction::Reset.to_vec(),
        };
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);
        let res = banks.process_transaction(tx).await;
        assert!(res.is_err());
    }
}

enum ClockState {
    Normal,
    TooEarly,
}

async fn setup_program_test_env(clock_state: ClockState) -> (BanksClient, Keypair, Keypair, Hash) {
    let mut program_test = ProgramTest::new("mars", mars::ID, processor!(mars::process_instruction));
    program_test.prefer_bpf(true);

    // Busses
    for i in 0..BUS_COUNT {
        program_test.add_account_with_base64_data(
            BUS_ADDRESSES[i],
            1057920,
            mars::id(),
            bs64::encode(
                &[
                    &(Bus::discriminator() as u64).to_le_bytes(),
                    Bus {
                        id: i as u64,
                        rewards: 0,
                    }
                    .to_bytes(),
                ]
                .concat(),
            )
            .as_str(),
        );
    }

    // Treasury
    let admin_address = Pubkey::from_str("staryJacbXodPh4WfwVtgA5jkJhvsMHERtkdttnLEHc").unwrap();
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &mars::id());
    program_test.add_account_with_base64_data(
        treasury_pda.0,
        1614720,
        mars::id(),
        bs64::encode(
            &[
                &(Treasury::discriminator() as u64).to_le_bytes(),
                Treasury {
                    bump: treasury_pda.1 as u64,
                    admin: admin_address,
                    difficulty: INITIAL_DIFFICULTY.into(),
                    last_reset_at: 0,
                    reward_rate: INITIAL_REWARD_RATE,
                    total_claimed_rewards: 0,
                }
                .to_bytes(),
            ]
            .concat(),
        )
        .as_str(),
    );

    // Mint
    let mut mint_src: [u8; Mint::LEN] = [0; Mint::LEN];
    Mint {
        mint_authority: COption::Some(TREASURY_ADDRESS),
        supply: 0,
        decimals: TOKEN_DECIMALS,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(&mut mint_src);
    program_test.add_account_with_base64_data(
        MINT_ADDRESS,
        1461600,
        spl_token::id(),
        bs64::encode(&mint_src).as_str(),
    );

    // Treasury tokens
    let tokens_address = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    let mut tokens_src: [u8; spl_token::state::Account::LEN] = [0; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint: MINT_ADDRESS,
        owner: TREASURY_ADDRESS,
        amount: 0,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut tokens_src);
    program_test.add_account_with_base64_data(
        tokens_address,
        2039280,
        spl_token::id(),
        bs64::encode(&tokens_src).as_str(),
    );

    // Set sysvar
    let ts = match clock_state {
        ClockState::Normal => START_AT + 1,
        ClockState::TooEarly => START_AT - 1,
    };
    program_test.add_sysvar_account(
        sysvar::clock::id(),
        &Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: DEFAULT_SLOTS_PER_EPOCH,
            unix_timestamp: ts,
        },
    );

    // Setup alt payer
    let payer_alt = Keypair::new();
    program_test.add_account(
        payer_alt.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (banks, payer, blockhash) = program_test.start().await;
    (banks, payer, payer_alt, blockhash)
}
