use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MarsError,
    instruction::ClaimArgs,
    loaders::*,
    state::{Proof, Treasury},
    utils::AccountDeserialize,
    MINT_ADDRESS, TREASURY,
};

/// Claim distributes owed token rewards from the treasury to the miner. Its responsibilies include:
/// 1. Transfer tokens from the treasury to the miner.
/// 2. Decrement the miner's claimable rewards counter by an appropriate amount.
/// 3. Update the program's lifetime stats.
///
/// Safety requirements:
/// - Claim is a permissionless instruction and can be called by any miner.
/// - Can only succeed if the claimed amount is less than or equal to the miner's claimable rewards.
/// - The provided beneficiary token account, treasury, treasury token account, and token program must be valid.
pub fn process_claim<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = ClaimArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, None, &MINT_ADDRESS, true)?;
    load_proof(proof_info, signer.key, true)?;
    load_treasury(treasury_info, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(treasury_info.key),
        &MINT_ADDRESS,
        true,
    )?;
    load_program(token_program, spl_token::id())?;

    // Update claimable amount
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.claimable_rewards = proof
        .claimable_rewards
        .checked_sub(amount)
        .ok_or(MarsError::ClaimTooLarge)?;

    // Update lifetime status
    let mut treasury_data = treasury_info.data.borrow_mut();
    let treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    treasury.total_claimed_rewards = treasury.total_claimed_rewards.saturating_add(amount);

    // Distribute tokens from treasury to beneficiary
    let treasury_bump = treasury.bump;
    drop(treasury_data);
    solana_program::program::invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            treasury_tokens_info.key,
            beneficiary_info.key,
            treasury_info.key,
            &[treasury_info.key],
            amount,
        )?,
        &[
            token_program.clone(),
            treasury_tokens_info.clone(),
            beneficiary_info.clone(),
            treasury_info.clone(),
        ],
        &[&[TREASURY, &[treasury_bump as u8]]],
    )?;

    Ok(())
}
