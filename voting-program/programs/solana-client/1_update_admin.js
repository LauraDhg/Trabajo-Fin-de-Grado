/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para actualizar los datos de la cuenta Admin al programa Voting
*/
import {
  Connection,
  Keypair,
  PublicKey,
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

const new_adminKeypair = new PublicKey("new_admin_Pub_key");
const new_adminBuffer = new_adminKeypair.publicKey.toBuffer(

);
// Datos de la instrucción SEGURA
const instructionData = Buffer.concat([
  Buffer.from([1]),             // Instruccion con indice 1 
  new_adminBuffer
]);

// Datos de la instrucción INSEGURA
/*
const instructionData = Buffer.concat([
  Buffer.from([1]), 
  new_adminBuffer,
  Buffer.from([adminBump])
]);
*/

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: adminPda, isSigner: false, isWritable: true },
    { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: instructionData,
});

// Crear y enviar transacción
const tx = new Transaction().add(instruction);

// Firmar transacción
const signature = await sendAndConfirmTransaction(connection, tx, [adminKeypair]);

console.log("Transaction signature:", signature);
