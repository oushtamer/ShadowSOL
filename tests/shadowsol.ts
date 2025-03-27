// import * as anchor from "@project-serum/anchor";
// import { Program } from "@project-serum/anchor";
// import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
// import * as splToken from "@solana/spl-token";
// import { assert } from "chai";
// import * as snarkjs from "snarkjs";
// import path from "path";
// import { buildPedersen } from "circomlibjs";
// import { buildBn128, utils } from "ffjavascript";

// const { unstringifyBigInts } = utils;

// describe("shadow_sol full zk‑flow", () => {
//   const provider = anchor.AnchorProvider.local();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.Shadowsol as Program;

//   let mint: PublicKey;
//   let userAta: PublicKey;
//   let progAta: PublicKey;
//   const user = provider.wallet.payer;
//   const denomination = new anchor.BN(1000);

//   const wasmPath = path.join(__dirname, "../circuits/withdraw.wasm");
//   const zkeyPath = path.join(__dirname, "../circuits/withdraw_final.zkey");

//   async function createMint(): Promise<PublicKey> {
//     return await splToken.createMint(
//       provider.connection,
//       provider.wallet.payer,
//       provider.wallet.publicKey,
//       null,
//       6
//     );
//   }

//   async function createAta(mint: PublicKey, owner: PublicKey): Promise<PublicKey> {
//     const ata = await splToken.getOrCreateAssociatedTokenAccount(
//       provider.connection,
//       provider.wallet.payer,
//       mint,
//       owner
//     );
//     return ata.address;
//   }

//   async function pedersenHash(nullifier: string, secret: string): Promise<Uint8Array> {
//     const ped = await buildPedersen();
//     const nullBuf = Buffer.from(nullifier, "hex");
//     const secBuf = Buffer.from(secret, "hex");
//     const hashBig = ped.hash(Buffer.concat([nullBuf, secBuf]));
//     const hex = hashBig.toString(16).padStart(64, "0");
//     return Uint8Array.from(hex.match(/.{2}/g)!.map(b => parseInt(b, 16)));
//   }

//   function serializeProof(proofJson: any): Uint8Array {
//     const ax = BigInt(proofJson.pi_a[0]).toString(16).padStart(64, "0");
//     const ay = BigInt(proofJson.pi_a[1]).toString(16).padStart(64, "0");
//     const a = hexToBytes(ax).concat(hexToBytes(ay));

//     const b0x = BigInt(proofJson.pi_b[0][0]).toString(16).padStart(64, "0");
//     const b0y = BigInt(proofJson.pi_b[0][1]).toString(16).padStart(64, "0");
//     const b1x = BigInt(proofJson.pi_b[1][0]).toString(16).padStart(64, "0");
//     const b1y = BigInt(proofJson.pi_b[1][1]).toString(16).padStart(64, "0");
//     const b = hexToBytes(b0x).concat(hexToBytes(b0y), hexToBytes(b1x), hexToBytes(b1y));

//     const cx = BigInt(proofJson.pi_c[0]).toString(16).padStart(64, "0");
//     const cy = BigInt(proofJson.pi_c[1]).toString(16).padStart(64, "0");
//     const c = hexToBytes(cx).concat(hexToBytes(cy));

//     return Uint8Array.from([...a, ...b, ...c]);
//   }

//   function hexToBytes(hex: string): number[] {
//     return hex.match(/.{2}/g)!.map(byte => parseInt(byte, 16));
//   }

//   function to32(num: any): Uint8Array {
//     const hex = BigInt(num).toString(16).padStart(64, "0");
//     return Uint8Array.from(hex.match(/.{2}/g)!.map(b => parseInt(b, 16)));
//   }

//   before(async () => {
//     mint = await createMint();
//     userAta = await createAta(mint, user.publicKey);
//     progAta = await createAta(mint, user.publicKey);
//     await splToken.mintTo(
//       provider.connection, provider.wallet.payer, mint, userAta, provider.wallet.payer, 10000
//     );
//   });

//   it("initialize → deposit → proof → withdraw", async () => {
//     const state = Keypair.generate();
//     await program.methods.initializeGlobal(denomination, mint)
//       .accounts({ shadowState: state.publicKey, signer: user.publicKey, systemProgram: SystemProgram.programId })
//       .signers([state]).rpc();

//     const nullifier = "0".repeat(64);
//     const secret    = "1".repeat(64);
//     const commitment = await pedersenHash(nullifier, secret);

//     const [commitPda] = await PublicKey.findProgramAddress(
//       [Buffer.from("commitment"), commitment], program.programId
//     );

//     await program.methods.deposit(Array.from(commitment))
//       .accounts({
//         shadowState: state.publicKey,
//         commitmentEntry: commitPda,
//         userTokenAccount: userAta,
//         programTokenAccount: progAta,
//         signer: user.publicKey,
//         tokenProgram: splToken.TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId
//       })
//       .rpc();

//     const { proof, publicSignals } = await snarkjs.groth16.fullProve(
//       { nullifier, secret, pathElements: ["0","0"], pathIndices: ["0","1"], root: publicSignals?.[0] || "0" },
//       wasmPath, zkeyPath
//     );

//     const proofBytes = serializeProof(unstringifyBigInts(proof));
//     const root = to32(publicSignals[0]);
//     const nullHash = to32(publicSignals[1]);
//     const recipient = to32(publicSignals[2]);
//     const token = to32(publicSignals[3]);

//     const [nullPda] = await PublicKey.findProgramAddress(
//       [Buffer.from("nullifier"), nullHash], program.programId
//     );

//     await program.methods.withdraw(Array.from(proofBytes), Array.from(root), Array.from(nullHash), Array.from(recipient), Array.from(token))
//       .accounts({
//         shadowState: state.publicKey,
//         nullifierEntry: nullPda,
//         programTokenAccount: progAta,
//         userTokenAccount: userAta,
//         programSigner: user.publicKey,
//         user: user.publicKey,
//         tokenProgram: splToken.TOKEN_PROGRAM_ID,
//         systemProgram: SystemProgram.programId
//       })
//       .rpc();

//     assert.equal(await splToken.getAccount(provider.connection, userAta).then(a => Number(a.amount)), 10000);
//     assert.equal(await splToken.getAccount(provider.connection, progAta).then(a => Number(a.amount)), 0);
//   });
// });
