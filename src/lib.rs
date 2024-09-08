use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program::{invoke},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

// Entry point of the program
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Get the player and game accounts
    let player_account = next_account_info(accounts_iter)?;
    let game_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Verify the accounts are owned by the program
    if player_account.owner != program_id || game_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Decode the instruction data
    if instruction_data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let stake_amount = u64::from_le_bytes(instruction_data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
    let player_choice = instruction_data[8];

    if player_choice > 1 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Check if the player's account has enough SOL to stake
    if **player_account.lamports.borrow() < stake_amount {
        msg!("Not enough SOL to stake");
        return Err(ProgramError::InsufficientFunds);
    }

    // Flip the coin (This is a placeholder; real randomness should be used)
    let coin_flip_result = rand::random::<bool>();
    let coin_flip_result_u8 = if coin_flip_result { 1 } else { 0 };

    if coin_flip_result_u8 == player_choice {
        msg!("Player won!");

        // Transfer double the stake back to the player
        let winnings = stake_amount.checked_mul(2).ok_or(ProgramError::InvalidInstructionData)?;
        if **game_account.lamports.borrow() < winnings {
            msg!("Game account does not have enough funds to pay the winnings");
            return Err(ProgramError::InsufficientFunds);
        }
        invoke(
            &system_instruction::transfer(game_account.key, player_account.key, winnings),
            &[
                game_account.clone(),
                player_account.clone(),
                system_program.clone(),
            ],
        )?;
    } else {
        msg!("Player lost!");

        // Transfer the staked SOL to the game account
        invoke(
            &system_instruction::transfer(player_account.key, game_account.key, stake_amount),
            &[
                player_account.clone(),
                game_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    Ok(())
}
