import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Crowdsale } from "../target/types/crowdsale";
import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import { BN } from "bn.js";
import { assert } from "chai";
import { createAccountHasBalance } from './helper'

const CROWDSALE_SEED = Buffer.from("crowdsale");
const CROWDSALE_TOKEN_VAULT_SEED = Buffer.from("crowdsale token vault");
const USER_INFO_SEED = Buffer.from("user account info");

describe("crowdsale", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;
  const connection = new Connection("http://127.0.0.1:8899", 'confirmed');


  const tokenMint = Keypair.fromSecretKey(new Uint8Array([
    71, 245, 98, 29, 212, 126, 135, 71, 146, 172, 76,
    177, 141, 23, 6, 92, 246, 182, 127, 233, 194, 224,
    225, 160, 84, 108, 90, 237, 33, 202, 6, 31, 90,
    91, 226, 190, 61, 93, 15, 65, 100, 47, 63, 229,
    246, 173, 0, 196, 127, 45, 102, 170, 111, 40, 55,
    120, 164, 45, 196, 24, 23, 157, 178, 133
  ]));

  // const tokenMint = Keypair.generate();

  // console.log(tokenMint.secretKey)

  const createTokenMint = async () => {
    const mint = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      9, // We are using 9 to match the CLI decimal default exactly
      tokenMint
    );

    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mint,
      payer.publicKey
    )
    await mintTo(
      connection,
      payer.payer,
      mint,
      tokenAccount.address,
      payer.publicKey,
      100_000_000 * LAMPORTS_PER_SOL // because decimals for the mint are set to 9 
    )
  }

  const program = anchor.workspace.Crowdsale as Program<Crowdsale>;

  let alice = Keypair.fromSecretKey(Uint8Array.from([
    176, 180, 119, 143, 117, 207, 1, 226, 60, 116, 179,
    92, 166, 83, 133, 245, 9, 136, 104, 12, 86, 14,
    204, 74, 89, 105, 97, 236, 145, 118, 161, 147, 23,
    62, 60, 156, 135, 2, 133, 222, 112, 159, 66, 203,
    8, 165, 187, 30, 99, 46, 131, 51, 215, 126, 229,
    26, 159, 5, 72, 229, 35, 181, 95, 5
  ]));
  let mark: Keypair;


  const [crowdsaleAccount] = PublicKey.findProgramAddressSync([CROWDSALE_SEED, payer.publicKey.toBuffer()], program.programId)
  const [crowdsaleTokenVaultAccount] = PublicKey.findProgramAddressSync([CROWDSALE_TOKEN_VAULT_SEED], program.programId)

  it("Create crowdsale account", async () => {
    // await createTokenMint()
    // alice = await createAccountHasBalance(connection, 10)
    // mark = await createAccountHasBalance(connection, 10)

    // const tx = await program.methods.createCrowdsale()
    //   .accounts({ crowdsaleAccount, crowdsaleTokenVaultAccount, mint: tokenMint.publicKey }).signers([payer.payer])
    //   .rpc();

    const crowdsaleAccountData = await program.account.crowdSale.fetch(crowdsaleAccount)

    console.log("Create crowdsale account: ", crowdsaleAccountData)

    // console.log("Your transaction signature", tx);
  });

  it("Setting crowdsale", async () => {
    const startTime = +(Date.now() / 1000).toFixed(0) + 10
    const endTime = startTime + 1000
    const minPrice = 0.2 * LAMPORTS_PER_SOL
    const maxPrice = 5 * LAMPORTS_PER_SOL
    const price = 0.2 * LAMPORTS_PER_SOL


    // await program.methods.settingCrowdsale(
    //   new BN(minPrice),
    //   new BN(maxPrice),
    //   new BN(startTime),
    //   new BN(endTime),
    //   new BN(price)
    // ).accounts({
    //   crowdsaleAccount,
    //   payer: payer.payer.publicKey,
    // }).signers([payer.payer]).rpc()



    const crowdsaleAccountData = await program.account.crowdSale.fetch(crowdsaleAccount)

    // assert(crowdsaleAccountData.price.toNumber() == price, "Price not match!")
    // assert(crowdsaleAccountData.maxPrice.toNumber() == maxPrice, "Max price not match!")
    // assert(crowdsaleAccountData.minPrice.toNumber() == minPrice, "Min price not match!")
    // assert(crowdsaleAccountData.start.toNumber() == startTime, "start time not match!")

    // console.log(crowdsaleAccountData.start.toNumber())
    // assert(crowdsaleAccountData.end.toNumber() == endTime, "end time not match!")

  })

  it("Deposit token to crowdsale", async () => {

    const amount = "70000000000000000";

    const userTokenAccount = await getOrCreateAssociatedTokenAccount(connection, payer.payer, tokenMint.publicKey, payer.payer.publicKey)

    // await program.methods
    //   .depositeToken(new anchor.BN(amount))
    //   .accounts({
    //     crowdsaleAccount,
    //     crowdsaleTokenVaultAccount,
    //     userTokenAccount: userTokenAccount.address,
    //     mint: tokenMint.publicKey,
    //     payer: payer.payer.publicKey
    //   }).signers([payer.payer]).rpc()


    const crowdsaleAccountData = await program.account.crowdSale.fetch(crowdsaleAccount)

    console.log("Deposite toke to crowdsale", crowdsaleAccountData)
  })

  it("Buy token", async () => {

    const crowdsaleAccountData = await program.account.crowdSale.fetch(crowdsaleAccount)

    const amount = LAMPORTS_PER_SOL / crowdsaleAccountData.price.toNumber() * 1 * LAMPORTS_PER_SOL;

    const aliceTokenAccount = await getOrCreateAssociatedTokenAccount(connection, alice, tokenMint.publicKey, alice.publicKey)

    const [aliceTokenInfoAccount] = PublicKey.findProgramAddressSync([USER_INFO_SEED, alice.publicKey.toBuffer()], program.programId)

    await program.methods.buyToken(payer.publicKey, new BN(1 * LAMPORTS_PER_SOL)).accounts({
      crowdsaleAccount,
      crowdsaleTokenVaultAccount,
      userTokenAccount: aliceTokenAccount.address,
      userInfoAccount: aliceTokenInfoAccount,
      mint: tokenMint.publicKey,
      payer: alice.publicKey
    }).signers([alice]).rpc()

    console.log((await program.account.userInfo.fetch(aliceTokenInfoAccount)).solAmount.toNumber())
    console.log(+aliceTokenAccount.amount.toString() == amount);

  })
});
