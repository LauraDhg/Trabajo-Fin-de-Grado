/*
Creado por: Laura De Haro García
Descripción: Maneja y desereliza correctamente las instrucciones incluidas en las transacciones.
*/
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub enum InstructionFormat {
    CreateAdmin     {},
    UpdateAdmin     {new_admin: Pubkey},
    CreateAccount   {amount_governance_points: u64},
    OpenProposal    {title: String, description: String},
    Vote            {vote_answer: bool},
    ManagePoints    {amount: u64, instruction:bool},
    ExecuteIntrs    {variant_instr: u8, amount:u64, vault_bump:u8} ,
    MakeDecision    {} ,
    
}

#[derive(BorshDeserialize, Debug)]
struct Update_Admin_Structure {
    new_admin: Pubkey
}
#[derive(BorshDeserialize, Debug)]
struct Create_Account_Structure {
    amount_governance_points: u64
}
#[derive(BorshDeserialize, Debug)]
struct OpenProposal_Structure {
    title: String,
    description: String,
}
#[derive(BorshDeserialize, Debug)]
struct Vote_Structure {
    vote_answer: bool
}
#[derive(BorshDeserialize, Debug)]
struct Points_Structure {
    amount: u64,
    instruction: bool,
}
#[derive(BorshDeserialize, Debug)]
struct Vault_Decisions_Structure {
   variant_instr: u8, 
   amount:u64,
   vault_bump: u8
}

impl InstructionFormat {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => {
                Self::CreateAdmin{}
            },
            1 => {
                let payload = Update_Admin_Structure::try_from_slice(rest)?;
                Self::UpdateAdmin{
                    new_admin: payload.new_admin
                }
            },
            2 => {
                let payload = Create_Account_Structure::try_from_slice(rest)?;
                Self::CreateAccount{
                    amount_governance_points: payload.amount_governance_points
                }
            },
            3 => {

                let payload: OpenProposal_Structure =OpenProposal_Structure::try_from_slice(rest)?;
                Self::OpenProposal {
                    title: payload.title,
                    description: payload.description
                }
            },
            4 => {
                let payload = Vote_Structure::try_from_slice(rest)?;
                Self::Vote{
                    vote_answer: payload.vote_answer
                }
            },
            5 => {
                let payload = Points_Structure::try_from_slice(rest)?;
                Self::ManagePoints{
                    amount: payload.amount,
                    instruction: payload.instruction
                }
            },
            6 => {
                let payload = Vault_Decisions_Structure::try_from_slice(rest)?;
                Self::ExecuteIntrs{
                    variant_instr: payload.variant_instr,
                    amount: payload.amount,
                    vault_bump: payload.vault_bump
                }
            },
            7 => {
                Self::MakeDecision{}
            },
            
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}