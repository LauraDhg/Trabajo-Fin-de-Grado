/*
Creado por: Laura De Haro García
Descripción: Script para mandar una trasancción con la instrucción para crear una cuenta de tipo Admin al programa Voting
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

// Datos de la instrucción SEGURA
const instructionData = Buffer.from([0]) // Instruccion con indice 0 

// Datos de la instrucción INSEGURA
// const instructionData = Buffer.from([0, adminBump])

// Crear instrucción
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },
    { pubkey: adminPda, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
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

