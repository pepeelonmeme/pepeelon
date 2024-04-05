import { Keypair, LAMPORTS_PER_SOL, Connection } from '@solana/web3.js';

export async function createAccountHasBalance (connection: Connection, amount: number) {
    const account = Keypair.generate()

    const airdropSignature = await connection.requestAirdrop(
        account.publicKey,
        amount * LAMPORTS_PER_SOL,
    );

    await connection.confirmTransaction(airdropSignature);

    console.log(account.secretKey)

    return account
}