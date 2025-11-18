/*
Creado por: Laura De Haro García
Descripción: "Program" para desplegar en la blockchain Solana, 
se trata de un programa simple que deja a un usuario crear una cuenta cartera, depositar, retirar y tranferir fondos ficticios.
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
    system_program
};
use borsh::{BorshDeserialize, BorshSerialize};

// Maneja la deserialización de las instrucciones incluidas en las transacciones
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
pub struct Wallet {
    pub amount: u64,
    pub initialized: bool,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = InstructionFormat::unpack(instruction_data)?;
    match instruction {
        InstructionFormat::CreateAccount {} => initialize_wallet_account(program_id, accounts),
        InstructionFormat::IncreaseAmount {amount} => deposit_amount(program_id, accounts, amount),
        InstructionFormat::WithdrawAmount {amount} => withdraw_amount(program_id, accounts, amount),
        InstructionFormat::TransferAmount {amount} => transfer_amount(program_id, accounts, amount), 
    }
}

// Crea una cuenta nueva de tipo Wallet.
// Cuentas pasadas: usuario de solana, PDA de la cuenta a crear, System Program.
pub fn initialize_wallet_account(
    program_id: &Pubkey, 
    accounts: &[AccountInfo]
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let wallet_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if system_program.key != &system_program::ID {
        return Err(ProgramError::InvalidArgument);
    }

    // Control de firma de la transacción
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Control de PDA correcto
    let (wallet_pda, wallet_bump) = Pubkey::find_program_address(&[b"wallet",user_account.key.as_ref()], program_id);
    if wallet_account.key != &wallet_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    let lamports = **wallet_account.lamports.borrow();
    if lamports > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let wallet = Wallet {
        amount: 0,
        initialized: true
    };

    let wallet_len = wallet.try_to_vec()?.len();
    let rent = Rent::get()?;
    let wallet_rent = rent.minimum_balance(wallet_len);

    invoke_signed(  
        &system_instruction::create_account(
            user_account.key,
            wallet_account.key,
            wallet_rent,
            wallet_len.try_into().unwrap(),
            program_id,
        ),
        &[
            user_account.clone(),
            wallet_account.clone(),
            system_program.clone(),
        ],
        &[&[b"wallet", user_account.key.as_ref(), &[wallet_bump]]],
    )?;

    msg!("Wallet account created");
    wallet.serialize(&mut &mut wallet_account.data.borrow_mut()[..])?;
    msg!("Wallet serialized");

    Ok(())
}

// Deposita una cantidad en la cuenta Wallet del usuario.
// Cuentas pasadas: usuario de solana, PDA de la cuenta Wallet
// Parámetros: Cantidad a depositar
pub fn deposit_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
) -> ProgramResult {
    msg!("Increasing amount by {}", amount);

    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let wallet_account = next_account_info(account_info_iter)?;

    // Control de propiedad de la cuenta
    if wallet_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Control de firma de la transacción
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Control de PDA correcto
    let (wallet_pda, wallet_bump) = Pubkey::find_program_address(&[b"wallet", user_account.key.as_ref()], program_id);
    if wallet_account.key != &wallet_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Unpacking wallet");
    let mut wallet = Wallet::try_from_slice(&wallet_account.data.borrow())?;
    msg!("Wallet unpacked: {:?}", wallet);

    // Control de cuenta correctamente inicializada
    if !wallet.initialized {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Current amount: {}", wallet.amount);
   
    // Adición de cantidad con control seguro
    wallet.amount = wallet.amount.checked_add(amount).ok_or(MyError::ArithmeticError)?;

    msg!("New amount: {}", wallet.amount);
    wallet.serialize(&mut &mut wallet_account.data.borrow_mut()[..])?;
    msg!("Wallet serialized");

    Ok(())
}

// Retira una cantidad de la cuenta Wallet del usuario.
// Cuentas pasadas: usuario de solana, PDA de la cuenta Wallet
// Parámetros: Cantidad a retirar
pub fn withdraw_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64
) -> ProgramResult {
    msg!("Withdraw from wallet {}", amount);

    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let wallet_account = next_account_info(account_info_iter)?;

    // Control de propiedad de la cuenta
    if wallet_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Control de firma de la transacción
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Control de PDA correcto
    let (wallet_pda, wallet_bump) = Pubkey::find_program_address(&[b"wallet", user_account.key.as_ref()], program_id);
    if wallet_account.key != &wallet_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Unpacking wallet");
    let mut wallet = Wallet::try_from_slice(&wallet_account.data.borrow())?;
    msg!("Wallet unpacked: {:?}", wallet);

    // Control de cuenta correctamente inicializada
    if !wallet.initialized {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Current amount: {}", wallet.amount);

    if wallet.amount < amount {
    msg!("Error: Not enough amount in wallet. Required: {}, Available: {}", amount, wallet.amount);
        return Err(ProgramError::InsufficientFunds); 
    }
    // Reducción de cantidad con control seguro
    wallet.amount = wallet.amount.checked_sub(amount).ok_or(MyError::ArithmeticError)?;
    
    msg!("New amount: {}", wallet.amount);

    wallet.serialize(&mut &mut wallet_account.data.borrow_mut()[..])?;
    msg!("Wallet serialized");

    Ok(())
}

// Transferir una cantidad de la cuenta Wallet de origen de un usario a la cuenta destino de otro.
// Cuentas pasadas: usuario origen de solana, usuario destino de solana, PDA de la cuenta Wallet de origen, PDA de la cuenta Wallet de destino
// Parámetros: Cantidad a transferir
pub fn transfer_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    msg!("Transfer amount {}", amount);

    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let src_wallet_account = next_account_info(account_info_iter)?;

    let dest_user_account = next_account_info(account_info_iter)?;
    let dest_wallet_account = next_account_info(account_info_iter)?;

    // Control de propiedad de la cuenta
    if src_wallet_account.owner != program_id || dest_wallet_account.owner != program_id{
        return Err(ProgramError::IllegalOwner);
    }

    // Control de firma de la transacción
    if !user_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Control de PDA correcto
    let (src_wallet_pda, src_wallet_bump) = Pubkey::find_program_address(&[b"wallet", user_account.key.as_ref()], program_id);
    let (dest_wallet_pda, dest_wallet_bump) = Pubkey::find_program_address(&[b"wallet", dest_user_account.key.as_ref()], program_id);
    if src_wallet_account.key != &src_wallet_pda || dest_wallet_account.key != &dest_wallet_pda {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Unpacking src wallet");
    let mut src_wallet = Wallet::try_from_slice(&src_wallet_account.data.borrow())?;
    msg!("Wallet unpacked: {:?}", src_wallet);
    msg!("Unpacking dest wallet");
    let mut dest_wallet = Wallet::try_from_slice(&dest_wallet_account.data.borrow())?;
    msg!("Wallet unpacked: {:?}", dest_wallet);

    // Control de cuenta correctamente inicializada
    if !src_wallet.initialized {
        return Err(ProgramError::InvalidAccountData);
    }
    if !dest_wallet.initialized {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("Current amount src wallet: {}", src_wallet.amount);
    msg!("Current amount dest wallet: {}", dest_wallet.amount);

    if src_wallet.amount < amount {
    msg!("Error: Not enough amount in wallet. Required: {}, Available: {}", amount, src_wallet.amount);
        return Err(ProgramError::InsufficientFunds); 
    }
    // Reducción de cantidad con control seguro
    src_wallet.amount = src_wallet.amount.checked_sub(amount).ok_or(MyError::ArithmeticError)?;
    // Adición de cantidad con control seguro
    dest_wallet.amount = dest_wallet.amount.checked_add(amount).ok_or(MyError::ArithmeticError)?;
    
    msg!("New amount src wallet: {}", src_wallet.amount);
    msg!("New amount dest wallet: {}", dest_wallet.amount);

    src_wallet.serialize(&mut &mut src_wallet_account.data.borrow_mut()[..])?;
    dest_wallet.serialize(&mut &mut dest_wallet_account.data.borrow_mut()[..])?;
    msg!("Both wallets serialized");

    Ok(())
}
