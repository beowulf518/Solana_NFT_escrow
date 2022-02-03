import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { AnchorEscrow } from '../target/types/anchor_escrow';
import { PublicKey, SystemProgram, Transaction, Connection } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { assert } from "chai";
import { getMultipleAccounts } from '@project-serum/anchor/dist/cjs/utils/rpc';
import { token } from '@project-serum/anchor/dist/cjs/utils';

describe('anchor-escrow', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;

  let mint = null;
  let vault_account_pda = null;
  let vault_account_bump = null;

  const price = 1500;
  const initializerAmount = 500000;

  const escrowAccount = anchor.web3.Keypair.generate();
  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const buyer = anchor.web3.Keypair.generate();

  const pdasTokenAccount = anchor.web3.Keypair.generate();
  const metaDataInfo = anchor.web3.Keypair.generate();
  const creatorAccWeb = anchor.web3.Keypair.generate();

  const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  );

  it("Initialize program state", async () => {
    // Airdropping tokens to a payer.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );

    // Fund Main Accounts
    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: initializerMainAccount.publicKey,
            lamports: 1000000000,
          }),
        );
        return tx;
      })(),
      [payer]
    );

    await provider.send(
      (() => {
        const tx = new Transaction();
        tx.add(
          SystemProgram.transfer({
            fromPubkey: payer.publicKey,
            toPubkey: buyer.publicKey,
            lamports: 1000000000,
          }),
        );
        return tx;
      })(),
      [payer]
    );

    mint = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );
  });

  it("Test List", async () => {
    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("token-seed")), escrowAccount.publicKey.toBuffer()],
        program.programId
      );
      vault_account_pda = _vault_account_pda;
      vault_account_bump = _vault_account_bump;

      let index = await (await program.account.escrowInfo.all()).length + 1;
  
      await program.rpc.listing(
        vault_account_bump,
        new anchor.BN(price),
        new anchor.BN(index),
        {
          accounts: {
            initializer: initializerMainAccount.publicKey,
            tokenAccount: vault_account_pda,
            mintKey: mint.publicKey,
            escrowAccount: escrowAccount.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          instructions: [
            await program.account.escrowInfo.createInstruction(escrowAccount),
          ],
          signers: [escrowAccount, initializerMainAccount],
        }
      );

      let escrows = await program.account.escrowInfo.all();
      escrows.forEach(element => {
          console.log(element);
      });
  });

  it("Test Buy", async () => {
    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode("escrow")), buyer.publicKey.toBuffer()],
        program.programId
      );
      vault_account_pda = _vault_account_pda;
      vault_account_bump = _vault_account_bump;
    
    let escrow = await program.account.escrowInfo.fetch(escrowAccount.publicKey);

    const [_vault_meta_pda, _vault_meta_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("metadata")), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID
    );
    let meta_info = _vault_meta_pda;

    await program.rpc.buy(
        vault_account_bump,
        new anchor.BN(price),
        {
          accounts: {
            buyer: buyer.publicKey,
            mintKey: mint.publicKey,
            escrowInfo: escrowAccount.publicKey,
            initializersMainAccount: initializerMainAccount.publicKey,
            pdasTokenAccount: escrow.tokenAccountPubkey,
            pdaAccount: vault_account_pda,
            metadataInfo: meta_info,
            tokenAccountAuthority: vault_account_pda,
            creatorAccWeb: creatorAccWeb.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetaProgram: TOKEN_METADATA_PROGRAM_ID,
          },
          signers: [buyer],
        }
      );
  });

  it("Test Cancel", async () => {
    let escrow = await program.account.escrowInfo.fetch(escrowAccount.publicKey);
    //console.log(escrow, vault_account_pda.toBase58());

    let token = await mint.getAccountInfo(vault_account_pda);
    console.log("token", token);
    console.log("token.owner", token.owner.toBase58(), "initializerMainAccount", initializerMainAccount.publicKey.toBase58());

    const [_vault_account_pda, _vault_account_bump] = await PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("escrow")), escrowAccount.publicKey.toBuffer()],
      program.programId
    );
    vault_account_pda = _vault_account_pda;
    vault_account_bump = _vault_account_bump;

    await program.rpc.cancel(
        {
          accounts: {
            user: initializerMainAccount.publicKey,
            escrowInfo: escrowAccount.publicKey,
            pdaAccount: vault_account_pda,
            pdasTokenAccount: escrow.tokenAccountPubkey,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers:[initializerMainAccount],
        }
      );
  });
});