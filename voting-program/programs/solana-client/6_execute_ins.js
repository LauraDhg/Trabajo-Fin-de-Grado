import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY
} from "@solana/web3.js";
import fs from "fs";

// Configuración
const PROGRAM_ID = new PublicKey("ID-PROGRAM");
const RPC_URL = "https://api.devnet.solana.com"; 
const connection = new Connection(RPC_URL, "confirmed");


// ---- Create and Withdraw instructions -----
// Claves del usuario Admin
const adminKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`user-wallet.json`, "utf8"))));

// Derivar PDA de la cuenta Admin SEGURO
const [adminPda, adminBump] = await PublicKey.findProgramAddress(
  [Buffer.from("admin")],
  PROGRAM_ID
);

// Derivar PDA de la cuenta Admin INSEGURO
/*
cons adminBump = 0; // Número entre 0-255 que sea válido
const seedsWithBump = [Buffer.from("admin"), Buffer.from([adminBump])];
const adminPda = await PublicKey.createProgramAddress(seedsWithBump, programId);
*/

// ---- Deposit instructions -----
/*
// Claves del usuario
const userKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`user-wallet.json`, "utf8"))));


// Derivar PDA de la cuenta User SEGURO
const [userPda, userBump] = await PublicKey.findProgramAddress(
  [Buffer.from("userAccount"), userKeypair.publicKey.toBuffer()],
  PROGRAM_ID
);
*/
// Derivar PDA de la cuenta User INSEGURO
/*
cons userBump = 0; // Número entre 0-255 que sea válido
const seedsWithBump = [Buffer.from("userAccount"), userKeypair.publicKey.toBuffer(), Buffer.from([userBump])];
const userPda = await PublicKey.createProgramAddress(seedsWithBump, programId);
*/

// Vault program Info
const vaultPda = new PublicKey("vault-pda-account");
const vault_program = new PublicKey("6dVLkpmd5BvVQF7oXpipnQFtKwba2uQkMMwzHnaqf8Gr");
  
const amount = 5;
const amountBuffer = Buffer.alloc(8);
amountBuffer.writeBigUInt64LE(BigInt(amount));

// Datos de la instrucción SEGURA
const instructionData = Buffer.concat([
  Buffer.from([6]),                             // Instruccion con indice 6
  Buffer.from([2]),                             // Instruccion para Vault program (0 - create / 1 - deposit / 2 - withdraw / 3 - message)
  amountBuffer ,
  Buffer.from([vaultBump])
]);

// Datos de la instrucción SEGURA
/*
const instructionData = Buffer.concat([
  Buffer.from([6]),                            
  Buffer.from([2]),                           
  amountBuffer ,
  Buffer.from([vaultBump]),
  Buffer.from([userBump])                        // o adminBump, depende del caso 
]);
*/


// ----- Instruccion 0 dentro de la transacción -----

const variant = 3;                               // Instruccion para Vault program (0 - create / 1 - deposit / 2 - withdraw / 3 - message)
const message = "REQUEST";

const instructionData_0 = Buffer.concat([
  Buffer.from([variant]), 
  Buffer.from(message, "utf-8")
]);

// Crear instrucción
const instruction_0 = new TransactionInstruction({
  keys: [
    { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },         // o userKeypair
  ],
  programId: vault_program,
  data: instructionData_0,
});


// ----- Instruccion 1 dentro de la transacción -----
const instruction_1 = new TransactionInstruction({
  keys: [
    { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },        // o userKeypair
    { pubkey: adminPda, isSigner: false, isWritable: true },                     // o userPda
    { pubkey: vaultPda, isSigner: false, isWritable: true },
    { pubkey: vault_program, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});


// Crear y enviar transacción
const tx = new Transaction().add(instruction_0,instruction_1);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [adminKeypair]);

console.log("Transaction signature:", signature);