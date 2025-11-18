/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para votar en una propouesta al programa Voting
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
import fs from "fs";

// Configuración
const PROGRAM_ID = new PublicKey("ID-PROGRAM");
const RPC_URL = "https://api.devnet.solana.com"; 
const connection = new Connection(RPC_URL, "confirmed");

// Claves del usuario
const userKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`user-wallet.json`, "utf8"))));

const userPda = new PublicKey("user-dao-account-pub-key");
const proposalPda = new PublicKey("proposal-dao-account-pub-key");

// Derivar PDA de la cuenta Vote_Record SEGURO
const [vote_record_Pda, voteBump] = await PublicKey.findProgramAddress(
  [Buffer.from("vote"), userPda.toBuffer(), proposalPda.toBuffer()],
    PROGRAM_ID
);

// Derivar PDA de la cuenta Vote_Record INSEGURO
/*
cons voteBump = 0; // Número entre 0-255 que sea válido
const seedsWithBump = [Buffer.from("vote"), userPda.toBuffer(), proposalPda.toBuffer(), Buffer.from([voteBump])];
const vote_record_Pda = await PublicKey.createProgramAddress(seedsWithBump, programId);
*/

const vote_answer = Buffer.alloc(1);        // 0 - no / 1 - yes
vote_answer[0] = 1;

// Datos de la instrucción SEGURA
const instructionData = Buffer.concat([
  Buffer.from([4]),                         // Instruccion con indice 4
  vote_answer
]);

// Datos de la instrucción INSEGURA
/*
const instructionData = Buffer.concat([
  Buffer.from([4]),
  vote_answer,
  Buffer.from([voteBump])
]);
*/

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: userPda, isSigner: false, isWritable: true },
    { pubkey: proposalPda, isSigner: false, isWritable: true },
    { pubkey: vote_record_Pda, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }    
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair]);

console.log("Transaction signature:", signature);