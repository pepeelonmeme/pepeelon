import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Crowdsale } from "../target/types/crowdsale";
import { Connection } from "@solana/web3.js";

describe("crowdsale", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const payer = provider.wallet as anchor.Wallet;
  const connection = new Connection("http://127.0.0.1:8899", 'confirmed');

  const program = anchor.workspace.Crowdsale as Program<Crowdsale>;

  it("Is createCrowdsale!", async () => {
    // Add your test here.
    const tx = await program.methods.createCrowdsale()
      .accounts({

      }).rpc();
    console.log("Your transaction signature", tx);
  });
});
