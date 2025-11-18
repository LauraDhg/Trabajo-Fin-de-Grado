/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para actualizar puntos de gobernanza de la cuenta de un usuario al programa Voting
*/
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} from "@solana/web3.js";
import fs from "fs";

// Configuración
const PROGRAM_ID = new PublicKey("ID-PROGRAM");
const RPC_URL = "https://api.devnet.solana.com"; 
const connection = new Connection(RPC_URL, "confirmed");

// Claves del usuario Admin
const adminKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`user-wallet.json`, "utf8"))));

const userPda = new PublicKey("user-dao-account-pub-key");

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

const amount = 30;
const amountBuffer = Buffer.alloc(8);
amountBuffer.writeBigUInt64LE(BigInt(amount));

// Datos de la instrucción SEGURA
const instructionData = Buffer.concat([
  Buffer.from([5]),                         // Instruccion con indice 4
  amountBuffer
]);

// Datos de la instrucción SEGURA
/*const instructionData = Buffer.concat([
  Buffer.from([5]), 
  amountBuffer ,
  Buffer.from([adminBump])
]);
*/

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: adminPda, isSigner: false, isWritable: true },
    { pubkey: userPda, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [adminKeypair]);

console.log("Transaction signature:", signature);