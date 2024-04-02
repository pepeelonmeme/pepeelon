import { clusterApiUrl, Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import base58 from "bs58";
import { privateKey } from "./key";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

const main = async () => {

    const connection = new Connection(clusterApiUrl("devnet"),
        "confirmed");

    const payer = Keypair.fromSecretKey(
        Uint8Array.from(base58.decode(privateKey))
    );

    const mint = await createMint(
        connection,
        payer,
        payer.publicKey,
        payer.publicKey,
        9
    );

    console.log("Mint: ", mint.toBase58());

    const tokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        payer,
        mint,
        payer.publicKey
    );

    await mintTo(
        connection,
        payer,
        mint,
        tokenAccount.address,
        payer.publicKey,
        100000000 * LAMPORTS_PER_SOL
    );
}

main();