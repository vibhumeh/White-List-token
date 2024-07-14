import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "bn.js";
import { assert } from "chai";
import {
  getAccount,
  getOrCreateAssociatedTokenAccount,
 
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { keypairIdentity, token, Metaplex } from "@metaplex-foundation/js";

describe('token_vault', () => {
  anchor.setProvider(anchor.AnchorProvider.env());


  const program = anchor.workspace.TokenVault;

  let mint = null;
  let tokenAccount = null;
  let tokenAccountOwnerPda = null;
  let whitelistPda = null;
  let vaultTokenAccount = null;
  const mintAuthority = program.wallet.keypair;
  const addressing = program.wallet.publicKey; //the address we are whitelisting, to keep things simple we are using our own address
  const decimals = 9;
  it('Initializes the program and creates a mint', async () => {
    const [counting] = await PublicKey.findProgramAddressSync(
      [addressing.toBuffer()],
      program.PROGRAM_ID
    );
    console.log("counter:"+counting);
    let [tokenAccountOwnerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_account_owner_pda")],
      program.PROGRAM_ID
    );
    const [whitelisted] = PublicKey.findProgramAddressSync(
      [Buffer.from("whitelist")],
      program.program.programId
    );
    
    const metaplex = new Metaplex(program.connection).use(
      keypairIdentity(program.wallet.keypair)
    );
    
    const createdSFT = await metaplex.nfts().createSft({
      uri: "https://shdw-drive.genesysgo.net/AzjHvXgqUJortnr5fXDG2aPkp2PfFMvu4Egr57fdiite/PirateCoinMeta",
      name: "Gold",
      symbol: "GOLD",
      sellerFeeBasisPoints: 100,
      updateAuthority: mintAuthority,
      mintAuthority: mintAuthority,
      decimals: decimals,
      //tokenStandard: "Fungible",
      isMutable: true,
    });
    
    console.log(
      "Creating semi fungible spl token with address: " + createdSFT.sft.address
    );
    
    const mintDecimals = Math.pow(10, decimals);
    
    let mintResult = await metaplex.nfts().mint({
      nftOrSft: createdSFT.sft,
      authority: program.wallet.keypair,
      toOwner: program.wallet.keypair.publicKey,
      amount: token(100 * mintDecimals),
    });
    
    console.log("Mint to result: " + mintResult.response.signature);//signature, will be dif np
    
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      program.connection,
      program.wallet.keypair,
      createdSFT.mintAddress,
      program.wallet.keypair.publicKey
    );
    
    console.log("tokenAccount: " + tokenAccount.address);
    console.log("TokenAccountOwnerPda: " + tokenAccountOwnerPda);
    
    let tokenAccountInfo = await getAccount(program.connection, tokenAccount.address);
    console.log(
      "Owned token amount: " + tokenAccountInfo.amount / BigInt(mintDecimals)
    );
    let [tokenVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_vault"), createdSFT.mintAddress.toBuffer()], //will create new vault for every new
      program.PROGRAM_ID
    );
    console.log("VaultAccount: " + tokenVault);
    
    let confirmOptions = {
      skipPreflight: true,
    };
    console.log("whitelist account: " + whitelisted);
    /////initialize
    
    ///--------
    
    let txHash = await program.program.methods
      .initialize()
      .accounts({
        tokenAccountOwnerPda: tokenAccountOwnerPda,
        vaultTokenAccount: tokenVault,
    
        //senderTokenAccount: tokenAccount.address,
        whitelist: whitelisted,
        mintOfTokenBeingSent: createdSFT.mintAddress,
        signer: program.wallet.publicKey,
      })
      .rpc(confirmOptions);
    
    console.log(`Initialize`);
});})
