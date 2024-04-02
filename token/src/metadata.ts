import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { Keypair, PublicKey } from "@solana/web3.js";
import base58 from "bs58";
import { metadata, privateKey, tokenMint } from "./key";
import { createSignerFromKeypair, none, signerIdentity } from "@metaplex-foundation/umi";
import { fromWeb3JsKeypair, fromWeb3JsPublicKey } from "@metaplex-foundation/umi-web3js-adapters";
import { Collection, createMetadataAccountV3, CreateMetadataAccountV3InstructionAccounts, CreateMetadataAccountV3InstructionDataArgs, Creator, Uses } from "@metaplex-foundation/mpl-token-metadata";

const main = async () => {

    const payer = Keypair.fromSecretKey(
        Uint8Array.from(base58.decode(privateKey))
    );

    const mint = new PublicKey(tokenMint);

    const umi = createUmi("https://api.devnet.solana.com");

    const signer = createSignerFromKeypair(umi, fromWeb3JsKeypair(payer));

    umi.use(signerIdentity(signer, true));

    const onChainData = {
        ...metadata,
        // we don't need that
        sellerFeeBasisPoints: 0,
        creators: none<Creator[]>(),
        collection: none<Collection>(),
        uses: none<Uses>(),
    }

    const accounts: CreateMetadataAccountV3InstructionAccounts = {
        mint: fromWeb3JsPublicKey(mint),
        mintAuthority: signer
    }

    const data: CreateMetadataAccountV3InstructionDataArgs = {
        isMutable: true,
        collectionDetails: null,
        data: onChainData
    }

    const txId = await createMetadataAccountV3(umi, { ...accounts, ...data }).sendAndConfirm(umi);

    console.log(txId)

}

main()