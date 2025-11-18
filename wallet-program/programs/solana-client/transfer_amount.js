/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para transferir de una cuenta a otra
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
const src_user = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(`src-user-wallet.json`, "utf8"))));

const dest_user = Keypair.fromSecretKey(
Uint8Array.from(JSON.parse(fs.readFileSync(`dest-user-wallet.json`, "utf8"))));

// Derivar PDA de la cuenta wallet 
const [src_walletPda, src_walletBump] = await PublicKey.findProgramAddress(
  [Buffer.from("wallet"),
    src_user.publicKey.toBuffer()],
  PROGRAM_ID
);

const [dest_walletPda, dest_walletBump] = await PublicKey.findProgramAddress(
  [Buffer.from("wallet"),
    dest_user.publicKey.toBuffer()],
  PROGRAM_ID
);

// Datos de la instrucción
const amount = 100;           // Cantidad a depositar
const instructionVariant = 3; // Instruccion con indice 3
const amountBuffer = Buffer.alloc(8);
amountBuffer.writeBigUInt64LE(BigInt(amount));

const instructionData = Buffer.concat([
  Buffer.from([instructionVariant]),
  amountBuffer
  
]);

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: src_user.publicKey, isSigner: true, isWritable: false },
    { pubkey: src_walletPda, isSigner: false, isWritable: true },
    { pubkey: dest_user.publicKey, isSigner: false, isWritable: false },
    { pubkey: dest_walletPda, isSigner: false, isWritable: true },
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [userKeypair]);

console.log("Transaction signature:", signature);