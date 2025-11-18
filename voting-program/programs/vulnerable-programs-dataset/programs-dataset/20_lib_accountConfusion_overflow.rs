use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::{ProgramResult},
    pubkey::Pubkey,
    msg,
    system_instruction,
    sysvar,
    sysvar::{rent::Rent, Sysvar},
    program::invoke_signed,
    borsh::try_from_slice_unchecked,
    hash::hashv,
    program_error::ProgramError,
    system_program  
};
use solana_program::sysvar::clock::{Clock, ID as CLOCK_SYSVAR_ID};
use solana_program::sysvar::instructions::{load_current_index,load_instruction_at};
use solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ID;
use borsh::{BorshDeserialize, BorshSerialize};
pub mod instructions;
use instructions::InstructionFormat;
use std::str::FromStr;
pub fn vault_program_id() -> Pubkey {
    Pubkey::from_str("6dVLkpmd5BvVQF7oXpipnQFtKwba2uQkMMwzHnaqf8Gr").unwrap()
}

pub enum MyError {
    TimeLimit = 0,
    ArithmeticError = 1
}


impl From<MyError> for ProgramError {
    fn from(e: MyError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Vote_Record {                                       
    pub proposal_id: Pubkey,
    pub vote_amount: u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct User_Account {                                  
    pub user_id: Pubkey,
    pub amount_governance_points: u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Admin_Account {
    pub admin_id: Pubkey,
    pub last_update: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Proposal {
    pub user_id: Pubkey,
    pub title: String,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub executed: bool,
    pub start_time: i64,
    pub end_time: i64,

}

fn action_as_str(action: u8) -> &'static str {
    match action {
        0 => "Create",
        1 => "Deposit",
        2 => "Withdraw",
        _ => "Unknown",
    }
}

fn send_create_vault_transaction<'info>( variant_instr: u8, id_pubkey: Pubkey, vault_bump: u8,
    admin_payer: &AccountInfo<'info>, admin_account_dao: &AccountInfo<'info>, vault_account: &AccountInfo<'info>, vault_program: &AccountInfo<'info>, system_program: &AccountInfo<'info>) -> ProgramResult{
    
    let mut ix_data: Vec<u8> = vec![variant_instr];
    ix_data.extend_from_slice(id_pubkey.as_ref());
    ix_data.extend_from_slice(&[vault_bump]); 

    let ix = solana_program::instruction::Instruction {
        program_id: *vault_program.key,
        accounts: vec![
            // Cuentas que el Vault Program espera
            solana_program::instruction::AccountMeta::new(*admin_payer.key, true),
            solana_program::instruction::AccountMeta::new(*admin_account_dao.key, false), 
            solana_program::instruction::AccountMeta::new(*vault_account.key, false),
            solana_program::instruction::AccountMeta::new_readonly(system_program.key.clone(), false),
        ],
        data: ix_data,
    };
    let vault_seeds: &[&[u8]] = &[b"vault", &[vault_bump]];

    invoke_signed(
        &ix,
        &[admin_payer.clone(),admin_account_dao.clone(), vault_account.clone(), system_program.clone()],
        &[vault_seeds],
    )?;

    Ok(())
}

fn send_instr_to_vault_transaction<'info>( variant_instr: u8, id_pubkey: Pubkey, amount: u64, vault_bump: u8,
    user_payer: &AccountInfo<'info>, user_account_dao: &AccountInfo<'info>, vault_account: &AccountInfo<'info>, vault_program: &AccountInfo<'info>) -> ProgramResult{
    
    let mut ix_data: Vec<u8> = vec![variant_instr];
    ix_data.extend_from_slice(&amount.to_le_bytes()); 
    ix_data.extend_from_slice(id_pubkey.as_ref());

    let ix = solana_program::instruction::Instruction {
        program_id: *vault_program.key,
        accounts: vec![
            // Cuentas que el Vault Program espera
            solana_program::instruction::AccountMeta::new(*user_payer.key, true),
            solana_program::instruction::AccountMeta::new(*user_account_dao.key, false), 
            solana_program::instruction::AccountMeta::new(*vault_account.key, false),
        ],
        data: ix_data,
    };
    let vault_seeds: &[&[u8]] = &[b"vault", &[vault_bump]];

    invoke_signed(
        &ix,
        &[user_payer.clone(),user_account_dao.clone(), vault_account.clone()],
        &[vault_seeds],
    )?;

    Ok(())
}

entrypoint!(process_instruction);


                fn process_instruction(
                program_id: &Pubkey,
                accounts: &[AccountInfo],
                instruction_data: &[u8],
            ) -> ProgramResult {
                let instruction = InstructionFormat::unpack(instruction_data)?;
                match instruction {
                    InstructionFormat::CreateAdmin   {} => create_admin(program_id, accounts),
                    InstructionFormat::UpdateAdmin   {new_admin} => update_admin(program_id, accounts, new_admin),
                    InstructionFormat::CreateAccount {} => create_account(program_id, accounts),
                    InstructionFormat::OpenProposal  {title, description} => open_proposal(program_id, accounts, title, description),
                    InstructionFormat::Vote          {vote_answer} => vote_process(program_id, accounts, vote_answer),
                    InstructionFormat::ManagePoints   {amount, instruction} => manage_governance_points(program_id, accounts, amount,instruction),
                    InstructionFormat::ExecuteIntrs  {variant_instr, amount, vault_bump} => execute_instruction(program_id, accounts, variant_instr, amount, vault_bump),
                    InstructionFormat::MakeDecision  {} => make_decision(program_id, accounts),
                }
            }
        
pub fn create_admin(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account_dao = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let clock_sysvar = next_account_info(account_info_iter)?;
    
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let lamports = **admin_account_dao.lamports.borrow();
    if lamports > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);
    
    if admin_account_dao.key != &admin_pda{
        msg!("Admin account incorrect");
        return Err(ProgramError::InvalidArgument);
    }

    if clock_sysvar.key != &CLOCK_SYSVAR_ID{
        msg!("Program not valid");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let account = Admin_Account {
        admin_id: *admin_payer.key,
        last_update: clock.unix_timestamp
    };

    let account_len = account.try_to_vec()?.len();
    let rent = Rent::get()?;
    let account_rent = rent.minimum_balance(account_len);
    invoke_signed(  
        &system_instruction::create_account(
            admin_payer.key,
            admin_account_dao.key,
            account_rent,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            admin_payer.clone(),
            admin_account_dao.clone(),
            system_program.clone(),
        ],
        &[&[b"admin", &[admin_bump]]],
    )?;

    msg!("Admin account created");
    msg!("Admin set: {}", admin_payer.key);
    account.serialize(&mut &mut admin_account_dao.data.borrow_mut()[..])?;
    msg!("Admin account serialized");

    Ok(())
}

pub fn update_admin(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
    new_admin: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account_dao = next_account_info(account_info_iter)?;
    let clock_sysvar = next_account_info(account_info_iter)?;
    
    if admin_account_dao.owner != program_id{
        return Err(ProgramError::IllegalOwner);
    }
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Unpacking admin account");
    let mut config_admin = Admin_Account::try_from_slice(&admin_account_dao.data.borrow())?;

    let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);

    if admin_account_dao.key != &admin_pda{
        msg!("Admin account incorrect");
        return Err(ProgramError::InvalidArgument);
    }
   
    if config_admin.admin_id != *admin_payer.key{
        msg!("User with no admin privileges");
        return Err(ProgramError::InvalidArgument);
    }
    
    if clock_sysvar.key != &CLOCK_SYSVAR_ID{
        msg!("Program not valid");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    msg!("Current admin: {}", config_admin.admin_id);
    msg!("Updating admin");
    config_admin.admin_id = new_admin;
    config_admin.last_update = clock.unix_timestamp;
    msg!("New admin: {}", new_admin);
    config_admin.serialize(&mut &mut admin_account_dao.data.borrow_mut()[..])?;

    Ok(())
}

pub fn create_account(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let user_payer = next_account_info(account_info_iter)?;
    let user_account_dao = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !user_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let lamports = **user_account_dao.lamports.borrow();
    if lamports > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (user_pda, user_bump) = Pubkey::find_program_address(&[b"userAccount", user_payer.key.as_ref()], program_id);
    
    if user_account_dao.key != &user_pda{
        msg!("user account incorrect");
        return Err(ProgramError::InvalidArgument);
    }
    
    let account = User_Account {

        user_id: *user_payer.key,
        amount_governance_points: 10
    };

    let account_len = account.try_to_vec()?.len();
    let rent = Rent::get()?;
    let account_rent = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            user_payer.key,
            user_account_dao.key,
            account_rent,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            user_payer.clone(),
            user_account_dao.clone(),
            system_program.clone(),
        ],
        &[&[b"userAccount", user_payer.key.as_ref(), &[user_bump]]],
    )?;

    msg!("User account created");
    msg!("User: {}", user_payer.key);
    msg!("Governance Points: {}", account.amount_governance_points);
    account.serialize(&mut &mut user_account_dao.data.borrow_mut()[..])?;
    msg!("User account serialized");

    Ok(())
}

pub fn open_proposal(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
    title: String,
    description: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let user_payer = next_account_info(account_info_iter)?;
    let user_account_dao = next_account_info(account_info_iter)?;
    let proposal_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let clock_sysvar = next_account_info(account_info_iter)?;

    if user_account_dao.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    if !user_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let lamports = **proposal_account.lamports.borrow();
    if lamports > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    } 

    let (proposal_pda, proposal_bump) = Pubkey::find_program_address(&[b"proposal",title.as_bytes()], program_id);
    
    if proposal_account.key != &proposal_pda{
        msg!("Proposal account incorrect");
        return Err(ProgramError::InvalidArgument);
    }
    

    if clock_sysvar.key != &CLOCK_SYSVAR_ID{
        msg!("Program not valid");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let start_time = clock.unix_timestamp;
    let end_time = start_time + 3600; // 1 hour later

    msg!("Title: {}", title.clone());
    msg!("Descr: {}", description.clone());
    msg!("User: {}", user_account_dao.key);
    let proposal = Proposal {
        user_id: *user_account_dao.key, 
        title: title.clone(),
        description: description,
        yes_votes: 0,
        no_votes: 0,
        executed: false,
        start_time: start_time,     
        end_time: end_time,   
    };

    let proposal_len = proposal.try_to_vec()?.len();
    let rent = Rent::get()?;
    let proposal_rent = rent.minimum_balance(proposal_len);

    invoke_signed(
        &system_instruction::create_account(
            user_payer.key,         
            proposal_account.key, 
            proposal_rent,
            proposal_len.try_into().unwrap(),
            program_id,
        ),
        &[
            user_payer.clone(),
            proposal_account.clone(),
            system_program.clone(),
        ],
        &[&[b"proposal", title.as_bytes(), &[proposal_bump]]],
    )?;

    msg!("Proposal created");
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;
    msg!("Proposal serialized");

    Ok(())
}

pub fn vote_process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    vote_answer: bool,
) -> ProgramResult {
    msg!("Vote Answer");
    let account_info_iter = &mut accounts.iter();

    let user_payer = next_account_info(account_info_iter)?;           
    let user_account_dao = next_account_info(account_info_iter)?;           
    let proposal_account = next_account_info(account_info_iter)?;     
    let vote_record_account = next_account_info(account_info_iter)?;  
    let system_program = next_account_info(account_info_iter)?;
    let clock_sysvar = next_account_info(account_info_iter)?;  

    if user_account_dao.owner != program_id 
        || proposal_account.owner != program_id {  
        return Err(ProgramError::IllegalOwner);
    }
    if !user_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let lamports = **vote_record_account.lamports.borrow();
    if lamports > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (vote_pda, vote_bump) = Pubkey::find_program_address(&[b"vote", user_account_dao.key.as_ref(), proposal_account.key.as_ref(),], program_id);
    
    if vote_record_account.key != &vote_pda{
        msg!("vote record account incorrect");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Unpacking proposal");
    let mut proposal = Proposal::try_from_slice(&proposal_account.data.borrow())?;
    msg!("Unpacking user account");
    let mut user = User_Account::try_from_slice(&user_account_dao.data.borrow())?;


    let vote_record = Vote_Record {

        proposal_id: *proposal_account.key,
        vote_amount: user.amount_governance_points
    };

    if clock_sysvar.key != &CLOCK_SYSVAR_ID{
        msg!("Program not valid");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let current_time = clock.unix_timestamp;
    if current_time > proposal.end_time{
        msg!("This proposal is no longer open");
        return Err(MyError::TimeLimit.into());
    } 

    if vote_answer {
        msg!("Current votes yes: {}", proposal.yes_votes);

	proposal.yes_votes += user.amount_governance_points;
        msg!("New votes yes: {}", proposal.yes_votes);

    } else {
        msg!("Current votes no: {}", proposal.no_votes);

	proposal.no_votes += user.amount_governance_points;
        msg!("New votes no: {}", proposal.no_votes);
    }

    let vote_len = vote_record.try_to_vec()?.len();
    let rent = Rent::get()?;
    let vote_rent = rent.minimum_balance(vote_len);

    invoke_signed(  
        &system_instruction::create_account(
            user_payer.key,
            vote_record_account.key,
            vote_rent,
            vote_len.try_into().unwrap(),
            program_id,
        ),
        &[
            user_payer.clone(),
            vote_record_account.clone(),
            system_program.clone(),
        ],
        &[&[b"vote", user_account_dao.key.as_ref(),proposal_account.key.as_ref(), &[vote_bump]]],
    )?;

    msg!("Vote record account created");
    msg!("Proposal: {}", proposal_account.key);
    msg!("Vote weigth: {}", user.amount_governance_points);
    vote_record.serialize(&mut &mut vote_record_account.data.borrow_mut()[..])?;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;
    user.serialize(&mut &mut user_account_dao.data.borrow_mut()[..])?;

    Ok(())
}

pub fn manage_governance_points(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    instruction: bool // add 1 // substract 0
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account_dao = next_account_info(account_info_iter)?;
    let user_account_dao = next_account_info(account_info_iter)?; 

    msg!("Add governance points to {}", user_account_dao.key);

    if admin_account_dao.owner != program_id || user_account_dao.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);
    
    if admin_account_dao.key != &admin_pda{
        msg!("Admin account incorrect");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Unpacking admin account");
    let mut admin = Admin_Account::try_from_slice(&admin_account_dao.data.borrow())?;
    if admin.admin_id != *admin_payer.key{
        msg!("User with no admin privileges");
        return Err(ProgramError::InvalidArgument);
    }
    admin.serialize(&mut &mut admin_account_dao.data.borrow_mut()[..])?;
  
    msg!("Unpacking user account");
    let mut user = User_Account::try_from_slice(&user_account_dao.data.borrow())?;

    
    msg!("Previous governance points: {}", user.amount_governance_points);
    if instruction{

	user.amount_governance_points += amount;
    } else{

        user.amount_governance_points = user.amount_governance_points.checked_sub(amount).ok_or(MyError::ArithmeticError)?;
    }
    msg!("New governance points: {}", user.amount_governance_points);
    user.serialize(&mut &mut user_account_dao.data.borrow_mut()[..])?;
    Ok(())
}

pub fn execute_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    variant_instr: u8, // 0 CREAR VAULT, 1 DEPOSITAR EN VAULT, 3 WITHDRAW FROM VAULT
    amount: u64,
    vault_bump: u8,
) -> ProgramResult {
    msg!("Execute instruction {}", action_as_str(variant_instr));

    let account_info_iter = &mut accounts.iter();

    let user_payer = next_account_info(account_info_iter)?;
    let user_account_dao = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;
    let vault_program = next_account_info(account_info_iter)?; 
    let system_program = next_account_info(account_info_iter)?; 
    let ix_sysvar = next_account_info(account_info_iter)?; 

    if user_account_dao.owner != program_id{
        return Err(ProgramError::IllegalOwner);
    }
    if !user_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let lamports = **user_account_dao.lamports.borrow();
    if lamports == 0 {
        msg!("This user account does not belong to the DAO program");
        return Err(ProgramError::InvalidArgument);
    }

    match variant_instr {
        0 =>{
            
            let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);
            
            if user_account_dao.key != &admin_pda{
                msg!("admin account incorrect");
                return Err(ProgramError::InvalidArgument);
            }
           
            msg!("Unpacking admin account");
            let mut admin = Admin_Account::try_from_slice(&user_account_dao.data.borrow())?;
            if admin.admin_id != *user_payer.key{
                msg!("User with no admin privileges");
                return Err(ProgramError::InvalidArgument);
            }
            admin.serialize(&mut &mut user_account_dao.data.borrow_mut()[..])?;

            Ok(())
        },
        1 => {
            
            let (user_pda, user__bump) = Pubkey::find_program_address(&[b"userAccount", user_payer.key.as_ref(),], program_id);
            
            if user_account_dao.key != &user_pda{
                msg!("user account incorrect");
                return Err(ProgramError::InvalidArgument);
            }
            Ok(())
        },
        2 => {
            
            let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);
            
            if user_account_dao.key != &admin_pda{
                msg!("admin account incorrect");
                return Err(ProgramError::InvalidArgument);
            }
            
            msg!("Unpacking admin account");
            let mut admin = Admin_Account::try_from_slice(&user_account_dao.data.borrow())?;
            if admin.admin_id != *user_payer.key{
                msg!("User with no admin privileges");
                return Err(ProgramError::InvalidArgument);
            }
            admin.serialize(&mut &mut user_account_dao.data.borrow_mut()[..])?;
            Ok(())
        },
        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    if vault_program.key != &vault_program_id() || ix_sysvar.key != &INSTRUCTIONS_SYSVAR_ID {
        msg!("Wrong Vault program");
        return Err(ProgramError::InvalidArgument);
    }

    let current_index = load_current_index(&ix_sysvar.try_borrow_data()?) ;
    msg!("current idx: {}", current_index);
    if current_index > 0 {
        let prev_ix = load_instruction_at((current_index - 1) as usize,&ix_sysvar.try_borrow_data()?).map_err(|_| ProgramError::InvalidInstructionData)?; 
        msg!("Prev instr program_id: {}", prev_ix.program_id);
        if prev_ix.program_id == vault_program_id() {
            msg!("The previous instruction was Vault program confirmation");
        }
    }

    let id_pubkey: Pubkey = program_id.clone();

    match variant_instr {
        0 => send_create_vault_transaction(variant_instr, id_pubkey, vault_bump, user_payer, user_account_dao, vault_account, vault_program, system_program),
        1 => send_instr_to_vault_transaction(variant_instr, id_pubkey, amount, vault_bump, user_payer, user_account_dao, vault_account, vault_program),
        2 => send_instr_to_vault_transaction(variant_instr, id_pubkey, amount, vault_bump, user_payer, user_account_dao, vault_account, vault_program),
        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    msg!("CPI ejecutada correctamente");
    Ok(())
}

pub fn make_decision(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("Execute instruction");
    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account_dao = next_account_info(account_info_iter)?;
    let proposal_account = next_account_info(account_info_iter)?; 
    let clock_sysvar = next_account_info(account_info_iter)?; 

    if admin_account_dao.owner != program_id || proposal_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (admin_pda, admin_bump) = Pubkey::find_program_address(&[b"admin"], program_id);
    
    if admin_account_dao.key != &admin_pda{
        msg!("Admin account incorrect");
        return Err(ProgramError::InvalidArgument);
    }
   
    msg!("Unpacking admin account");
    let mut admin = Admin_Account::try_from_slice(&admin_account_dao.data.borrow())?;
    if admin.admin_id != *admin_payer.key{
        msg!("User with no admin privileges");
        return Err(ProgramError::InvalidArgument);
    }
    admin.serialize(&mut &mut admin_account_dao.data.borrow_mut()[..])?;

    msg!("Unpacking proposal");
    let mut proposal = Proposal::try_from_slice(&proposal_account.data.borrow())?;

    if clock_sysvar.key != &CLOCK_SYSVAR_ID{
        msg!("Program not valid");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let current_time = clock.unix_timestamp;
    if (current_time < proposal.end_time){
        msg!("The proposal is still active");
        return Err(MyError::TimeLimit.into());
    }

    msg!("Votes yes: {} vs votes no {}", proposal.yes_votes, proposal.no_votes);
    if (proposal.yes_votes >= proposal.no_votes){
        msg!("Proposal approved");
    } else{
        msg!("Proposal denied"); 
    }
    proposal.executed = true;
    proposal.serialize(&mut &mut proposal_account.data.borrow_mut()[..])?;

    Ok(())
}