//! Program state processor
use crate::{
    error::LotteryError,
    instruction::LotteryInstruction,
    state::{LotteryData, LotteryResultData, TicketData},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    native_token::sol_to_lamports,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

// Sollotto program_id
solana_program::declare_id!("urNhxed8ocNiFApoooLSAJ1xnWSMUiC9S6fKcRon1rk");

/// Checks that the supplied program ID is the correct
pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// Program state handler.
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        check_program_account(program_id)?;

        let instruction = LotteryInstruction::unpack(instruction_data)?;
        match instruction {
            LotteryInstruction::InitLottery {
                lottery_id,
                charity_1,
                charity_2,
                charity_3,
                charity_4,
                holding_wallet,
                rewards_wallet,
                slot_holders_rewards_wallet,
                sollotto_labs_wallet,
            } => {
                msg!("Instruction: InitLottery");
                Self::process_init_lottery(
                    program_id,
                    accounts,
                    lottery_id,
                    charity_1,
                    charity_2,
                    charity_3,
                    charity_4,
                    holding_wallet,
                    rewards_wallet,
                    slot_holders_rewards_wallet,
                    sollotto_labs_wallet,
                )
            }

            LotteryInstruction::PurchaseTicket {
                charity,
                user_wallet_pk,
                ticket_number_arr,
            } => {
                msg!("Instruction: PurchaseTicket");
                Self::process_ticket_purchase(
                    program_id,
                    accounts,
                    charity,
                    user_wallet_pk,
                    ticket_number_arr,
                )
            }

            LotteryInstruction::StoreWinningNumbers {
                winning_numbers_arr,
            } => {
                msg!("Instruction: store winning numbers");
                Self::process_store_winning_numbers(program_id, accounts, winning_numbers_arr)
            }

            LotteryInstruction::RewardWinners {} => {
                msg!("Instruction: reward winners");
                Self::process_reward_winners(program_id, accounts)
            }

            LotteryInstruction::UpdateCharity {
                charity_1,
                charity_2,
                charity_3,
                charity_4,
            } => {
                msg!("Instrction: update charity");
                Self::process_update_charity(
                    program_id, accounts, charity_1, charity_2, charity_3, charity_4,
                )
            }

            LotteryInstruction::UpdateSollottoWallets {
                holding_wallet,
                rewards_wallet,
                slot_holders_rewards_wallet,
                sollotto_labs_wallet,
            } => {
                msg!("Instruction: update sollotto wallets");
                Self::process_update_sollotto_wallets(
                    program_id,
                    accounts,
                    holding_wallet,
                    rewards_wallet,
                    slot_holders_rewards_wallet,
                    sollotto_labs_wallet,
                )
            }
        }
    }

    pub fn process_init_lottery(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        lottery_id: u32,
        charity_1: Pubkey,
        charity_2: Pubkey,
        charity_3: Pubkey,
        charity_4: Pubkey,
        holding_wallet: Pubkey,
        rewards_wallet: Pubkey,
        slot_holders_rewards_wallet: Pubkey,
        sollotto_labs_wallet: Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        // lottery data account
        let lottery_data_account = next_account_info(accounts_iter)?;

        // Check if program owns data account
        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
        if !rent.is_exempt(
            lottery_data_account.lamports(),
            lottery_data_account.data_len(),
        ) {
            return Err(LotteryError::NotRentExempt.into());
        }

        // Add data to account
        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        if lottery_data.is_initialized {
            return Err(LotteryError::Initialized.into());
        }

        lottery_data.is_initialized = true;
        lottery_data.lottery_id = lottery_id;
        lottery_data.charity_1 = charity_1;
        lottery_data.charity_2 = charity_2;
        lottery_data.charity_3 = charity_3;
        lottery_data.charity_4 = charity_4;
        lottery_data.charity_1_vc = 0;
        lottery_data.charity_2_vc = 0;
        lottery_data.charity_3_vc = 0;
        lottery_data.charity_4_vc = 0;
        lottery_data.holding_wallet = holding_wallet;
        lottery_data.rewards_wallet = rewards_wallet;
        lottery_data.slot_holders_rewards_wallet = slot_holders_rewards_wallet;
        lottery_data.sollotto_labs_wallet = sollotto_labs_wallet;
        lottery_data.total_registrations = 0;
        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        msg!("Data stored");

        Ok(())
    }

    pub fn process_ticket_purchase(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        charity: Pubkey,
        user_wallet_pk: Pubkey,
        ticket_number_arr: [u8; 6],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let lottery_data_account = next_account_info(accounts_iter)?;
        let ticket_data_account = next_account_info(accounts_iter)?;
        let user_funding_account = next_account_info(accounts_iter)?;
        let holding_wallet_account = next_account_info(accounts_iter)?;
        let rent = &Rent::from_account_info(next_account_info(accounts_iter)?)?;
        let system_program_info = next_account_info(accounts_iter)?;

        // Check if program owns lottery data account
        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        //Check if lottery initisalised
        if !lottery_data.is_initialized {
            return Err(LotteryError::NotInitialized.into());
        }

        // Check if program owns ticket data account
        if ticket_data_account.owner != program_id {
            msg!("Ticket Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !rent.is_exempt(
            lottery_data_account.lamports(),
            lottery_data_account.data_len(),
        ) {
            return Err(LotteryError::NotRentExempt.into());
        }

        if !rent.is_exempt(
            ticket_data_account.lamports(),
            ticket_data_account.data_len(),
        ) {
            return Err(LotteryError::NotRentExempt.into());
        }

        let mut ticket_data = TicketData::unpack_unchecked(&ticket_data_account.data.borrow())?;
        if ticket_data.is_purchased {
            return Err(LotteryError::AlreadyPurchased.into());
        }

        for i in 0..5 {
            if ticket_number_arr[i] < 1 || ticket_number_arr[i] > 69 {
                return Err(LotteryError::InvalidNumber.into());
            }
        }
        if ticket_number_arr[5] < 1 || ticket_number_arr[5] > 29 {
            return Err(LotteryError::InvalidNumber.into());
        }

        ticket_data.is_purchased = true;
        ticket_data.charity = charity;
        ticket_data.user_wallet_pk = user_wallet_pk;
        ticket_data.ticket_number_arr = ticket_number_arr;

        lottery_data.total_registrations += 1;
        let charity_arr = [
            lottery_data.charity_1,
            lottery_data.charity_2,
            lottery_data.charity_3,
            lottery_data.charity_4,
        ];
        msg!("Charity Ids: {:?}", charity_arr);
        for (pos, key) in charity_arr.iter().enumerate() {
            msg!("Entered Loop");
            msg!("Current Charity: {}", *key);
            msg!("Receieved Charity: {}", charity);
            if *key == charity {
                msg!("Matched ID Loop");
                match pos {
                    0 => lottery_data.charity_1_vc += 1,
                    1 => lottery_data.charity_2_vc += 1,
                    2 => lottery_data.charity_3_vc += 1,
                    3 => lottery_data.charity_4_vc += 1,
                    _ => return Err(LotteryError::InvalidCharity.into()),
                }
                break;
            }
        }

        // Transfer 0.1 SOL into holding wallet from user_wallet
        let ticket_price = sol_to_lamports(0.01);
        invoke(
            &system_instruction::transfer(
                &user_wallet_pk,
                &lottery_data.holding_wallet,
                ticket_price,
            ),
            &[
                user_funding_account.clone(),
                holding_wallet_account.clone(),
                system_program_info.clone(),
            ],
        )?;

        TicketData::pack(ticket_data, &mut ticket_data_account.data.borrow_mut())?;
        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_store_winning_numbers(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        winning_numbers_arr: [u8; 6],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let lottery_data_account = next_account_info(accounts_iter)?;

        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        if !lottery_data.is_initialized {
            msg!("Lottery Data account is not initialized");
            return Err(LotteryError::NotInitialized.into());
        }

        for i in 0..5 {
            if winning_numbers_arr[i] < 1 || winning_numbers_arr[i] > 69 {
                return Err(LotteryError::InvalidNumber.into());
            }
        }
        if winning_numbers_arr[5] < 1 || winning_numbers_arr[5] > 29 {
            return Err(LotteryError::InvalidNumber.into());
        }

        lottery_data.winning_numbers = winning_numbers_arr;

        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_reward_winners(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let lottery_data_account = next_account_info(accounts_iter)?;
        let lottery_result_account = next_account_info(accounts_iter)?;
        let participants_tickets_accounts = accounts_iter.as_slice();

        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }
        if lottery_result_account.owner != program_id {
            msg!("Lottery Result Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        for participant_ticket in participants_tickets_accounts {
            if participant_ticket.owner != program_id {
                msg!("Ticket Data account does not have the correct program id");
                return Err(ProgramError::IncorrectProgramId);
            }
        }

        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        if !lottery_data.is_initialized {
            msg!("Lottery Data account is not initialized");
            return Err(LotteryError::NotInitialized.into());
        }

        // TODO: more than 1 winner
        // Check winning numbers and find winner
        let mut winner = Pubkey::default();
        for participant_ticket in participants_tickets_accounts {
            let ticket = TicketData::unpack_unchecked(&participant_ticket.data.borrow())?;
            if ticket.ticket_number_arr == lottery_data.winning_numbers {
                winner = ticket.user_wallet_pk;
            }
        }

        // TODO: Reward winner and pay fee
        // 6. The charity with the most votes is transferred 30% of the total prize pool
        // 7. 4% of the prize pool is transferred to the "Sollotto Rewards" wallet address
        // 8. 0.6% of the prize pool is transferred to a "SLOT Holder Rewards" wallet address
        // 9. 0.4% of the prize pool is transferred to a "Sollotto Labs" wallet address
        // 14. If there is just one winner, transfer the remaining portion (64%) of the prize pool to the
        // winner

        // Create lottery result acc info
        let mut lottery_result =
            LotteryResultData::unpack_unchecked(&lottery_result_account.data.borrow())?;
        lottery_result.lottery_id = lottery_data.lottery_id;
        lottery_result.winner = winner;
        LotteryResultData::pack(
            lottery_result,
            &mut lottery_result_account.data.borrow_mut(),
        )?;

        // Clear lottery acc for new lottery
        lottery_data.is_initialized = false;
        lottery_data.charity_1_vc = 0;
        lottery_data.charity_2_vc = 0;
        lottery_data.charity_3_vc = 0;
        lottery_data.charity_4_vc = 0;
        lottery_data.winning_numbers = [0, 0, 0, 0, 0, 0];
        lottery_data.total_registrations = 0;
        lottery_data.lottery_id = 0;
        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_update_charity(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        charity_1: Pubkey,
        charity_2: Pubkey,
        charity_3: Pubkey,
        charity_4: Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let lottery_data_account = next_account_info(accounts_iter)?;

        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        if !lottery_data.is_initialized {
            msg!("Lottery Data account is not initialized");
            return Err(LotteryError::NotInitialized.into());
        }

        lottery_data.charity_1 = charity_1;
        lottery_data.charity_2 = charity_2;
        lottery_data.charity_3 = charity_3;
        lottery_data.charity_4 = charity_4;

        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_update_sollotto_wallets(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        holding_wallet: Pubkey,
        rewards_wallet: Pubkey,
        slot_holders_rewards_wallet: Pubkey,
        sollotto_labs_wallet: Pubkey,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let lottery_data_account = next_account_info(accounts_iter)?;

        if lottery_data_account.owner != program_id {
            msg!("Lottery Data account does not have the correct program id");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !lottery_data_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut lottery_data = LotteryData::unpack_unchecked(&lottery_data_account.data.borrow())?;
        if !lottery_data.is_initialized {
            msg!("Lottery Data account is not initialized");
            return Err(LotteryError::NotInitialized.into());
        }

        lottery_data.holding_wallet = holding_wallet;
        lottery_data.rewards_wallet = rewards_wallet;
        lottery_data.slot_holders_rewards_wallet = slot_holders_rewards_wallet;
        lottery_data.sollotto_labs_wallet = sollotto_labs_wallet;

        LotteryData::pack(lottery_data, &mut lottery_data_account.data.borrow_mut())?;

        Ok(())
    }
}

// Unit tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::{instruction::Instruction, program_pack::Pack};
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
        ReadableAccount,
    };

    fn lottery_minimum_balance() -> u64 {
        Rent::default().minimum_balance(LotteryData::get_packed_len())
    }

    fn ticket_minimum_balance() -> u64 {
        Rent::default().minimum_balance(TicketData::get_packed_len())
    }

    fn do_process(instruction: Instruction, accounts: Vec<&mut SolanaAccount>) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    #[test]
    fn test_init_lottery() {
        let program_id = id();
        let lottery_id = 112233;
        let lottery_key = Pubkey::new_unique();
        let mut lottery_acc = SolanaAccount::new(
            lottery_minimum_balance(),
            LotteryData::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_acc = create_account_for_test(&Rent::default());
        let charity_1 = Pubkey::new_unique();
        let charity_2 = Pubkey::new_unique();
        let charity_3 = Pubkey::new_unique();
        let charity_4 = Pubkey::new_unique();
        let holding_wallet = Pubkey::new_unique();
        let rewards_wallet = Pubkey::new_unique();
        let slot_holders_rewards_wallet = Pubkey::new_unique();
        let sollotto_labs_wallet = Pubkey::new_unique();

        // BadCase: rent NotRentExempt
        let mut bad_lottery_acc = SolanaAccount::new(
            lottery_minimum_balance() - 100,
            LotteryData::get_packed_len(),
            &program_id,
        );
        assert_eq!(
            Err(LotteryError::NotRentExempt.into()),
            do_process(
                crate::instruction::initialize_lottery(
                    &program_id,
                    lottery_id,
                    &charity_1,
                    &charity_2,
                    &charity_3,
                    &charity_4,
                    &holding_wallet,
                    &rewards_wallet,
                    &slot_holders_rewards_wallet,
                    &sollotto_labs_wallet,
                    &lottery_key
                )
                .unwrap(),
                vec![&mut bad_lottery_acc, &mut rent_sysvar_acc]
            )
        );

        do_process(
            crate::instruction::initialize_lottery(
                &program_id,
                lottery_id,
                &charity_1,
                &charity_2,
                &charity_3,
                &charity_4,
                &holding_wallet,
                &rewards_wallet,
                &slot_holders_rewards_wallet,
                &sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc, &mut rent_sysvar_acc],
        )
        .unwrap();

        // BadCase: Lottery Already initialized
        assert_eq!(
            Err(LotteryError::Initialized.into()),
            do_process(
                crate::instruction::initialize_lottery(
                    &program_id,
                    lottery_id,
                    &charity_1,
                    &charity_2,
                    &charity_3,
                    &charity_4,
                    &holding_wallet,
                    &rewards_wallet,
                    &slot_holders_rewards_wallet,
                    &sollotto_labs_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc, &mut rent_sysvar_acc]
            )
        );

        let lottery = LotteryData::unpack(&lottery_acc.data).unwrap();
        assert_eq!(lottery.is_initialized, true);
        assert_eq!(lottery.lottery_id, lottery_id);
        assert_eq!(lottery.charity_1, charity_1);
        assert_eq!(lottery.charity_2, charity_2);
        assert_eq!(lottery.charity_3, charity_3);
        assert_eq!(lottery.charity_4, charity_4);
        assert_eq!(lottery.charity_1_vc, 0);
        assert_eq!(lottery.charity_2_vc, 0);
        assert_eq!(lottery.charity_3_vc, 0);
        assert_eq!(lottery.charity_4_vc, 0);
        assert_eq!(lottery.holding_wallet, holding_wallet);
        assert_eq!(lottery.rewards_wallet, rewards_wallet);
        assert_eq!(
            lottery.slot_holders_rewards_wallet,
            slot_holders_rewards_wallet
        );
        assert_eq!(lottery.sollotto_labs_wallet, sollotto_labs_wallet);
        assert_eq!(lottery.total_registrations, 0);
        for number in &lottery.winning_numbers {
            assert_eq!(*number, 0);
        }
    }

    #[test]
    fn test_ticket_purchase() {
        let program_id = id();
        let lottery_id = 112233;
        let lottery_key = Pubkey::new_unique();
        let mut lottery_acc = SolanaAccount::new(
            lottery_minimum_balance(),
            LotteryData::get_packed_len(),
            &program_id,
        );
        let user_funding_key = Pubkey::new_unique();
        let mut user_funding_acc = SolanaAccount::default();
        let user_ticket_key = Pubkey::new_unique();
        let mut user_ticket_acc = SolanaAccount::new(
            ticket_minimum_balance(),
            TicketData::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_acc = create_account_for_test(&Rent::default());
        let mut system_acc = SolanaAccount::default();
        let charity_1 = Pubkey::new_unique();
        let charity_2 = Pubkey::new_unique();
        let charity_3 = Pubkey::new_unique();
        let charity_4 = Pubkey::new_unique();
        let holding_wallet = Pubkey::new_unique();
        let mut holding_wallet_acc = SolanaAccount::default();
        let rewards_wallet = Pubkey::new_unique();
        let slot_holders_rewards_wallet = Pubkey::new_unique();
        let sollotto_labs_wallet = Pubkey::new_unique();
        let user_charity = charity_1;

        // BadCase: Lottery is not initialized
        assert_eq!(
            Err(LotteryError::NotInitialized.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[10, 20, 30, 40, 50, 15],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut user_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );

        do_process(
            crate::instruction::initialize_lottery(
                &program_id,
                lottery_id,
                &charity_1,
                &charity_2,
                &charity_3,
                &charity_4,
                &holding_wallet,
                &rewards_wallet,
                &slot_holders_rewards_wallet,
                &sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc, &mut rent_sysvar_acc],
        )
        .unwrap();

        // BadCase: rent NotRentExempt
        let mut bad_ticket_acc = SolanaAccount::new(
            ticket_minimum_balance() - 100,
            TicketData::get_packed_len(),
            &program_id,
        );
        assert_eq!(
            Err(LotteryError::NotRentExempt.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[10, 20, 30, 40, 50, 15],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut bad_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );

        // BadCase: bad numbers
        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[70, 20, 30, 40, 50, 15],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut user_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );

        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[10, 20, 30, 40, 0, 15],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut user_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );

        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[10, 20, 30, 40, 50, 30],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut user_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );

        do_process(
            crate::instruction::purchase_ticket(
                &program_id,
                &user_charity,
                &user_funding_key,
                &[10, 20, 30, 40, 50, 29],
                &user_ticket_key,
                &holding_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![
                &mut lottery_acc,
                &mut user_ticket_acc,
                &mut user_funding_acc,
                &mut holding_wallet_acc,
                &mut rent_sysvar_acc,
                &mut system_acc,
            ],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.charity_1_vc, 1);
        assert_eq!(lottery.total_registrations, 1);

        // BadCase: Ticket already purchased
        assert_eq!(
            Err(LotteryError::AlreadyPurchased.into()),
            do_process(
                crate::instruction::purchase_ticket(
                    &program_id,
                    &user_charity,
                    &user_funding_key,
                    &[10, 20, 30, 40, 50, 30],
                    &user_ticket_key,
                    &holding_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![
                    &mut lottery_acc,
                    &mut user_ticket_acc,
                    &mut user_funding_acc,
                    &mut holding_wallet_acc,
                    &mut rent_sysvar_acc,
                    &mut system_acc
                ]
            )
        );
    }

    #[test]
    fn test_store_winning_numbers() {
        let program_id = id();
        let lottery_id = 112233;
        let lottery_key = Pubkey::new_unique();
        let mut lottery_acc = SolanaAccount::new(
            lottery_minimum_balance(),
            LotteryData::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_acc = create_account_for_test(&Rent::default());
        let charity_1 = Pubkey::new_unique();
        let charity_2 = Pubkey::new_unique();
        let charity_3 = Pubkey::new_unique();
        let charity_4 = Pubkey::new_unique();
        let holding_wallet = Pubkey::new_unique();
        let rewards_wallet = Pubkey::new_unique();
        let slot_holders_rewards_wallet = Pubkey::new_unique();
        let sollotto_labs_wallet = Pubkey::new_unique();

        // BadCase: Lottery is not initialized
        assert_eq!(
            Err(LotteryError::NotInitialized.into()),
            do_process(
                crate::instruction::store_winning_numbers(
                    &program_id,
                    &[10, 20, 30, 40, 50, 29],
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        do_process(
            crate::instruction::initialize_lottery(
                &program_id,
                lottery_id,
                &charity_1,
                &charity_2,
                &charity_3,
                &charity_4,
                &holding_wallet,
                &rewards_wallet,
                &slot_holders_rewards_wallet,
                &sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc, &mut rent_sysvar_acc],
        )
        .unwrap();

        // BadCase: Bad numbers
        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::store_winning_numbers(
                    &program_id,
                    &[70, 20, 30, 40, 50, 29],
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::store_winning_numbers(
                    &program_id,
                    &[10, 20, 30, 40, 0, 29],
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        assert_eq!(
            Err(LotteryError::InvalidNumber.into()),
            do_process(
                crate::instruction::store_winning_numbers(
                    &program_id,
                    &[10, 20, 30, 40, 50, 30],
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        do_process(
            crate::instruction::store_winning_numbers(
                &program_id,
                &[10, 20, 30, 40, 50, 29],
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.winning_numbers[0], 10);
        assert_eq!(lottery.winning_numbers[1], 20);
        assert_eq!(lottery.winning_numbers[2], 30);
        assert_eq!(lottery.winning_numbers[3], 40);
        assert_eq!(lottery.winning_numbers[4], 50);
        assert_eq!(lottery.winning_numbers[5], 29);
    }

    #[test]
    fn test_update_charity() {
        let program_id = id();
        let lottery_id = 112233;
        let lottery_key = Pubkey::new_unique();
        let mut lottery_acc = SolanaAccount::new(
            lottery_minimum_balance(),
            LotteryData::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_acc = create_account_for_test(&Rent::default());
        let charity_1 = Pubkey::new_unique();
        let charity_2 = Pubkey::new_unique();
        let charity_3 = Pubkey::new_unique();
        let charity_4 = Pubkey::new_unique();
        let holding_wallet = Pubkey::new_unique();
        let rewards_wallet = Pubkey::new_unique();
        let slot_holders_rewards_wallet = Pubkey::new_unique();
        let sollotto_labs_wallet = Pubkey::new_unique();

        let new_charity_1 = Pubkey::new_unique();
        let new_charity_2 = charity_1;
        let new_charity_3 = Pubkey::new_unique();
        let new_charity_4 = charity_4;

        // BadCase: Lottery is not initialized
        assert_eq!(
            Err(LotteryError::NotInitialized.into()),
            do_process(
                crate::instruction::update_charity(
                    &program_id,
                    &new_charity_1,
                    &new_charity_2,
                    &new_charity_3,
                    &new_charity_4,
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        do_process(
            crate::instruction::initialize_lottery(
                &program_id,
                lottery_id,
                &charity_1,
                &charity_2,
                &charity_3,
                &charity_4,
                &holding_wallet,
                &rewards_wallet,
                &slot_holders_rewards_wallet,
                &sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc, &mut rent_sysvar_acc],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.charity_1, charity_1);
        assert_eq!(lottery.charity_2, charity_2);
        assert_eq!(lottery.charity_3, charity_3);
        assert_eq!(lottery.charity_4, charity_4);

        do_process(
            crate::instruction::update_charity(
                &program_id,
                &new_charity_1,
                &new_charity_2,
                &new_charity_3,
                &new_charity_4,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.charity_1, new_charity_1);
        assert_eq!(lottery.charity_2, new_charity_2);
        assert_eq!(lottery.charity_3, new_charity_3);
        assert_eq!(lottery.charity_4, new_charity_4);
    }

    #[test]
    fn test_update_sollotto_wallets() {
        let program_id = id();
        let lottery_id = 112233;
        let lottery_key = Pubkey::new_unique();
        let mut lottery_acc = SolanaAccount::new(
            lottery_minimum_balance(),
            LotteryData::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_acc = create_account_for_test(&Rent::default());
        let charity_1 = Pubkey::new_unique();
        let charity_2 = Pubkey::new_unique();
        let charity_3 = Pubkey::new_unique();
        let charity_4 = Pubkey::new_unique();
        let holding_wallet = Pubkey::new_unique();
        let rewards_wallet = Pubkey::new_unique();
        let slot_holders_rewards_wallet = Pubkey::new_unique();
        let sollotto_labs_wallet = Pubkey::new_unique();

        let new_holding_wallet = Pubkey::new_unique();
        let new_rewards_wallet = rewards_wallet;
        let new_slot_holders_rewards_wallet = Pubkey::new_unique();
        let new_sollotto_labs_wallet = sollotto_labs_wallet;

        // BadCase: Lottery is not initialized
        assert_eq!(
            Err(LotteryError::NotInitialized.into()),
            do_process(
                crate::instruction::update_sollotto_wallets(
                    &program_id,
                    &new_holding_wallet,
                    &new_rewards_wallet,
                    &new_slot_holders_rewards_wallet,
                    &new_sollotto_labs_wallet,
                    &lottery_key,
                )
                .unwrap(),
                vec![&mut lottery_acc]
            )
        );

        do_process(
            crate::instruction::initialize_lottery(
                &program_id,
                lottery_id,
                &charity_1,
                &charity_2,
                &charity_3,
                &charity_4,
                &holding_wallet,
                &rewards_wallet,
                &slot_holders_rewards_wallet,
                &sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc, &mut rent_sysvar_acc],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.holding_wallet, holding_wallet);
        assert_eq!(lottery.rewards_wallet, rewards_wallet);
        assert_eq!(
            lottery.slot_holders_rewards_wallet,
            slot_holders_rewards_wallet
        );
        assert_eq!(lottery.sollotto_labs_wallet, sollotto_labs_wallet);

        do_process(
            crate::instruction::update_sollotto_wallets(
                &program_id,
                &new_holding_wallet,
                &new_rewards_wallet,
                &new_slot_holders_rewards_wallet,
                &new_sollotto_labs_wallet,
                &lottery_key,
            )
            .unwrap(),
            vec![&mut lottery_acc],
        )
        .unwrap();

        let lottery = LotteryData::unpack(&lottery_acc.data()).unwrap();
        assert_eq!(lottery.holding_wallet, new_holding_wallet);
        assert_eq!(lottery.rewards_wallet, new_rewards_wallet);
        assert_eq!(
            lottery.slot_holders_rewards_wallet,
            new_slot_holders_rewards_wallet
        );
        assert_eq!(lottery.sollotto_labs_wallet, new_sollotto_labs_wallet);
    }
}
