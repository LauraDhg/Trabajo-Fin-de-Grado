/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para crear una cuenta de tipo Propuesta al programa Voting
*/
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import fs from "fs"
import * as borsh from "borsh";

class OpenProposalInstruction {
  constructor(props) {
    this.title = props.title; 
    this.description = props.description;
  }
}

const OpenProposalSchema = new Map([
  [OpenProposalInstruction, {
    kind: 'struct',
    fields: [
      ['title', 'string'],
      ['description', 'string']
    ],
  }],
]);

const title = "PROPOSAL 1";
const description = "Something to vote for ";

// Configuración
const PROGRAM_ID = new PublicKey("ID-PROGRAM");
const RPC_URL = "https://api.devnet.solana.com"; 
const connection = new Connection(RPC_URL, "confirmed");

// Claves del usuario
const userKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`user-wallet.json`, "utf8"))));

const userPda = new PublicKey("user-dao-account-pub-key");

// Derivar PDA de la cuenta Propouesta SEGURO
const [proposalPda, proposalBump] = await PublicKey.findProgramAddress(
  [Buffer.from("proposal"), Buffer.from(title)],
  PROGRAM_ID
);

// Derivar PDA de la cuenta Propuesta INSEGURO
/*
cons proposalBump = 0; // Número entre 0-255 que sea válido
const seedsWithBump = [Buffer.from("proposal"), Buffer.from(title), Buffer.from([proposalBump])];
const proposalPda = await PublicKey.createProgramAddress(seedsWithBump, programId);
*/

const instructionData = new OpenProposalInstruction({
  title,
  description,
});
const instr_serialized = borsh.serialize(OpenProposalSchema, instructionData);

// Datos de la instrucción SEGURA
const final_data = Buffer.concat([
  Buffer.from([3]),     // Instruccion con indice 3 
  instr_serialized
]);

// Datos de la instrucción INSEGURA
/*
const final_data = Buffer.concat([
  Buffer.from([3]),
  instr_serialized,
  Buffer.from([proposalBump]),
]);
*/

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: userPda, isSigner: false, isWritable: true },
    { pubkey: proposalPda, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }    
  ],
  programId: PROGRAM_ID,
  data:final_data,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair]);

console.log("Transaction signature:", signature);
