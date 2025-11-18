/*
Creado por: Laura De Haro García
Descripción: Maneja y desereliza correctamente las instrucciones incluidas en las transacciones.
*/
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub enum InstructionFormat {
    CreateVault {program_caller_id: Pubkey, vault_bump:u8},
    IncreaseAmount {amount: u64, program_caller_id: Pubkey},
    WithdrawAmount {amount: u64, program_caller_id: Pubkey},
    LogAction   {message: String}

}


#[derive(BorshDeserialize, Debug)]
struct Instr_struct {
    amount: u64,
    program_caller_id: Pubkey
   
}

#[derive(BorshDeserialize, Debug)]
struct Instr_struct_create {
    program_caller_id: Pubkey,
    vault_bump: u8
}

#[derive(BorshDeserialize, Debug)]
struct Log_struct {
    message:String
}

impl InstructionFormat {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {
                let payload = Instr_struct_create::try_from_slice(rest)?;
                Self::CreateVault{
                   program_caller_id: payload.program_caller_id,
                   vault_bump: payload.vault_bump,
                }
            },
            1 => {
                let payload = Instr_struct::try_from_slice(rest)?;
                Self::IncreaseAmount {
                    amount: payload.amount,
                    program_caller_id: payload.program_caller_id,
                }
            },
            2 => {
                let payload = Instr_struct::try_from_slice(rest)?;
                Self::WithdrawAmount {
                    amount: payload.amount,
                    program_caller_id: payload.program_caller_id,
                }
            },
            3 => {
                let message_payload = std::str::from_utf8(rest).map_err(|_| ProgramError::InvalidInstructionData)?;
                Self::LogAction {
                    message: message_payload.to_string(),
                }
            },
            
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}