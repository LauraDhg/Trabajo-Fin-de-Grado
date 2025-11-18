/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para crear una cuenta de tipo User al programa Voting
*/
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
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


// Derivar PDA de la cuenta User SEGURO
const [userPda, userBump] = await PublicKey.findProgramAddress(
  [Buffer.from("userAccount"), userKeypair.publicKey.toBuffer()],
  PROGRAM_ID
);

// Derivar PDA de la cuenta User INSEGURO
/*
cons userBump = 0; // Número entre 0-255 que sea válido
const seedsWithBump = [Buffer.from("userAccount"), userKeypair.publicKey.toBuffer(), Buffer.from([userBump])];
const userPda = await PublicKey.createProgramAddress(seedsWithBump, programId);
*/

const instructionVariant = 2;           // Instruccion con indice 2 
const amount = 100;                     // Para causar desbordamineto usar (1n << 64n) - 199n;
const amountBuffer = Buffer.alloc(8);  
amountBuffer.writeBigUInt64LE(BigInt(amount));  

// Datos de la instrucción SEGURA
const instructionData = Buffer.concat([
  Buffer.from([instructionVariant]), 
  amountBuffer
]);

// Datos de la instrucción INSEGURA
/*
const instructionData = Buffer.concat([
  Buffer.from([instructionVariant]), 
  amountBuffer,                   
  Buffer.from([userBump])
]);
*/

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: userKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: userPda, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair]);

console.log("Transaction signature:", signature);