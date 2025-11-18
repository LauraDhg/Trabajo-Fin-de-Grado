/*
Creado por: Laura De Haro García
Descripción: Maneja y desereliza correctamente las instrucciones incluidas en las transacciones.
*/
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum InstructionFormat {
    CreateAccount {},
    IncreaseAmount { amount: u64 },
    WithdrawAmount { amount: u64 },
    TransferAmount { amount: u64 }

}

#[derive(BorshDeserialize, Debug)]
struct Instr_struct {
    amount: u64
}

impl InstructionFormat {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {Self::CreateAccount {}
            },
            1 => {
                let payload = Instr_struct::try_from_slice(rest)?;
                Self::IncreaseAmount {
                    amount: payload.amount
                }
            },
            2 => {
                let payload = Instr_struct::try_from_slice(rest)?;
                Self::WithdrawAmount {
                    amount: payload.amount
                }
            },
            3 => {
                let payload = Instr_struct::try_from_slice(rest)?;
                Self::TransferAmount{
                    amount: payload.amount,
                }
            },
            
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}