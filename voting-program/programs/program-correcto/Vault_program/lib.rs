/*
Creado por: Laura De Haro García
Descripción: "Program" para desplegar en la blockchain Solana, 
se trata de un programa simple maneja las instrucciones que le envía el programa Voting con relación a 
crear un vault comunitario, depositar fondos o retirar fondos.
*/
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    program::invoke_signed,
    borsh::try_from_slice_unchecked,
    system_program,
    program_error::ProgramError,
};
use borsh::{BorshDeserialize, BorshSerialize};

// Maneja la deserialización de las instrucciones incluidas en las transacciones.
pub mod instructions;
use instructions::InstructionFormat;

pub enum MyError {
    ArithmeticError,
}

impl From<MyError> for ProgramError {
    fn from(e: MyError) -> Self {
        match e {
            MyError::ArithmeticError => ProgramError::Custom(1),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Community_Vault {
    pub amount: u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Admin_Account {
    pub admin_id: Pubkey,
    pub last_update: i64, 
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = InstructionFormat::unpack(instruction_data)?;
    match instruction {
        InstructionFormat::CreateVault {program_caller_id, vault_bump} => initialize_community_vault(program_id, accounts, &program_caller_id, vault_bump),
        InstructionFormat::IncreaseAmount {amount,program_caller_id} => deposit_amount(program_id, accounts,amount, &program_caller_id),
        InstructionFormat::WithdrawAmount {amount,program_caller_id} => withdraw_amount(program_id, accounts,amount, &program_caller_id),
        InstructionFormat::LogAction {message} => log_message(program_id, accounts,message),
    }
}

// Crea una cuenta nueva de tipo Community_Vault.
// Cuentas pasadas: usuario de solana (admin), PDA de la cuenta Admin_Account, 
// PDA de la cuenta Community_Vault, System Program
pub fn initialize_community_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    program_caller_id: &Pubkey,
    vault_bump: u8
) -> ProgramResult {
    msg!("Initialize community vault");
    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?; 
    let system_program = next_account_info(account_info_iter)?;

    if system_program.key != &system_program::ID || admin_account.owner != program_caller_id{
        return Err(ProgramError::IllegalOwner);
    }
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Caller: Voting program: {}", program_caller_id);

    let vault = Community_Vault {
        amount: 0
    };

    let vault_len = vault.try_to_vec()?.len();
    let rent = Rent::get()?;
    let vault_rent = rent.minimum_balance(vault_len);

    invoke_signed(
        &system_instruction::create_account(
            admin_payer.key,
            vault_account.key,
            vault_rent,
            vault_len.try_into().unwrap(),
            program_id,
        ),
        &[
            admin_payer.clone(),
            vault_account.clone(),
            system_program.clone(),
        ],
         &[&[b"vault", &[vault_bump]]],
    )?;

    msg!("Vault initialized");
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    Ok(())
}

// Deposita fondos en la cuenta Community_Vault.
// Cuentas pasadas: usuario de solana, PDA de la cuenta User_Account, 
// PDA de la cuenta Community_Vault
pub fn deposit_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    program_caller_id: &Pubkey
) -> ProgramResult {
    msg!("Increment amount by {}", amount);

    let account_info_iter = &mut accounts.iter();

    let user_payer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;

    if user_account.owner != program_caller_id || vault_account.owner != program_id{
        return Err(ProgramError::IllegalOwner);
    }
    if !user_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let lamports = **user_account.lamports.borrow();
    if lamports == 0 {
        msg!("This user account does not belong to the DAO program");
        return Err(ProgramError::InvalidArgument);
    }

    let mut vault  = Community_Vault::try_from_slice(&vault_account.data.borrow())?;
    msg!("vault unpacked: {:?}", vault);

    msg!("Current amount: {}", vault.amount);
    vault.amount = vault.amount.checked_add(amount).ok_or(MyError::ArithmeticError)?;
    msg!("New amount: {}", vault.amount);

    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;
    msg!("Vault serialized");

    Ok(())
}

// Retira fondos de la cuenta Community_Vault.
// Cuentas pasadas: usuario de solana (admin), PDA de la cuenta Admin_Account, 
// PDA de la cuenta Community_Vault
pub fn withdraw_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    program_caller_id: &Pubkey,
) -> ProgramResult {
    msg!("Withdraw from vault {}", amount);

    let account_info_iter = &mut accounts.iter();

    let admin_payer = next_account_info(account_info_iter)?;
    let admin_account = next_account_info(account_info_iter)?;
    let vault_account = next_account_info(account_info_iter)?;

    if admin_accountt.owner != program_caller_id || vault_account.owner != program_id{
        return Err(ProgramError::IllegalOwner);
    }
    if !admin_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let mut vault  = Community_Vault::try_from_slice(&vault_account.data.borrow())?;
    msg!("vault unpacked: {:?}", vault);

    if vault.amount < amount {
    msg!("Error: Not enough amount in wallet. Required: {}, Available: {}", amount, vault.amount);
        return Err(ProgramError::InsufficientFunds); 
    }
    msg!("Current amount: {}", vault.amount);
    vault.amount = vault.amount.checked_sub(amount).ok_or(MyError::ArithmeticError)?;
    msg!("New amount: {}", vault.amount);

    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;
    msg!("Vault serialized");

    Ok(())
}


// Mensaje de confimación para realizar una instrucción.
pub fn log_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    message: String
) -> ProgramResult {
    msg!("Action approved by vault authority");
    Ok(())
}