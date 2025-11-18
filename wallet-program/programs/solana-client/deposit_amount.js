/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para depositar en una cuenta al programa Wallet
*/
import {
  Connection,
  Keypair,
  PublicKey,
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

// Derivar PDA de la cuenta wallet 
const [walletPda, walletBump] = await PublicKey.findProgramAddress(
  [Buffer.from("wallet"),
    userKeypair.publicKey.toBuffer()],
  PROGRAM_ID
);

// Datos de la instrucción
const amount = 100;           // Cantidad a depositar
const instructionVariant = 1; // Instruccion con indice 1
const amountBuffer = Buffer.alloc(8);
amountBuffer.writeBigUInt64LE(BigInt(amount));

const instructionData = Buffer.concat([
  Buffer.from([instructionVariant]),
  amountBuffer
  
]);

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: userKeypair.publicKey, isSigner: true, isWritable: false },
    { pubkey: walletPda, isSigner: false, isWritable: true }
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair]);

console.log("Transaction signature:", signature);
