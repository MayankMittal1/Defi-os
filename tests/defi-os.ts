import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import * as spl from "@solana/spl-token";
import { DefiOs } from "../target/types/defi_os";

const {
  Connection,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  PublicKey,
  SystemProgram,
} = anchor.web3;

describe("Defi-OS ", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();

  const program = anchor.workspace.DefiOs as Program<DefiOs>;

  anchor.setProvider(provider);
  const repoAccount = anchor.web3.Keypair.generate();
  let exchange_token_mint: anchor.web3.Keypair;
  let mint: anchor.web3.Keypair;
  let user_exchange_token_account: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;
  const [repoVault, vault_bump] = await PublicKey.findProgramAddress(
    [
      anchor.utils.bytes.utf8.encode("repo-vault"),
      repoAccount.publicKey.toBuffer(),
    ],
    program.programId
  );
  const [repoVaultUSDC, vaultBumpUSDC] = await PublicKey.findProgramAddress(
    [
      anchor.utils.bytes.utf8.encode("repo-treasury"),
      repoAccount.publicKey.toBuffer(),
    ],
    program.programId
  );
  it("Mints Exchange Tokens to the user wallet", async () => {
    exchange_token_mint = anchor.web3.Keypair.generate();
    let create_mint_tx = new Transaction().add(
      // create mint account
      SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: exchange_token_mint.publicKey,
        space: spl.MintLayout.span,
        lamports: await spl.getMinimumBalanceForRentExemptMint(
          program.provider.connection
        ),
        programId: spl.TOKEN_PROGRAM_ID,
      }),
      // init mint account
      spl.createInitializeMintInstruction(
        exchange_token_mint.publicKey,
        6,
        provider.wallet.publicKey,
        provider.wallet.publicKey,
        spl.TOKEN_PROGRAM_ID
      )
    );

    await program.provider.sendAndConfirm(create_mint_tx, [
      exchange_token_mint,
    ]);

    user_exchange_token_account = await spl.getAssociatedTokenAddress(
      exchange_token_mint.publicKey,
      provider.wallet.publicKey,
      false,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );

    let create_user_exchange_token_account_tx = new Transaction().add(
      spl.createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        user_exchange_token_account,
        provider.wallet.publicKey,
        exchange_token_mint.publicKey,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      )
    );

    await program.provider.sendAndConfirm(
      create_user_exchange_token_account_tx
    );
    let mint_tokens_tx = new Transaction().add(
      spl.createMintToInstruction(
        exchange_token_mint.publicKey, // mint
        user_exchange_token_account, // receiver (sholud be a token account)
        provider.wallet.publicKey, // mint authority
        1e5,
        [], // only multisig account will use.
        spl.TOKEN_PROGRAM_ID
      )
    );

    await program.provider.sendAndConfirm(mint_tokens_tx);

    console.log(
      "Exchange Token balance: ",
      await program.provider.connection.getTokenAccountBalance(
        user_exchange_token_account
      )
    );
  });

  it("Initializes Repo Tokens and accounts", async () => {
    console.log(repoAccount.publicKey.toBase58());
    mint = anchor.web3.Keypair.generate();
    await program.methods
      .initializeRepo("ABCD", vault_bump, vaultBumpUSDC)
      .accounts({
        exchangeTokenMint: exchange_token_mint.publicKey,
        mint: mint.publicKey,
        repoAccount: repoAccount.publicKey,
        repoVault: repoVault,
        repoVaultUsdc: repoVaultUSDC,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([repoAccount, mint])
      .rpc();

    let account = await program.account.repository.fetch(repoAccount.publicKey);

    console.log("Repository: ", account);
    console.log(
      "Repo Token balance: ",
      await program.provider.connection.getTokenAccountBalance(repoVault)
    );
  });

  it("Buys Tokens", async () => {
    userTokenAccount = await spl.getAssociatedTokenAddress(
      mint.publicKey,
      provider.wallet.publicKey,
      false,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );

    let create_user_token_account_tx = new Transaction().add(
      spl.createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        userTokenAccount,
        provider.wallet.publicKey,
        mint.publicKey,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      )
    );

    await program.provider.sendAndConfirm(create_user_token_account_tx);
    console.log(repoAccount.publicKey.toBase58());
    await program.methods
      .buyTokens(new anchor.BN(20000), vault_bump)
      .accounts({
        exchangeTokenMint: exchange_token_mint.publicKey,
        repoAccount: repoAccount.publicKey,
        repoVault: repoVault,
        //repoVaultUsdc: repoVaultUSDC,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        userExchangeTokenAccount: user_exchange_token_account,
        userTokenAccount: userTokenAccount,
      })
      .rpc();
  });
});
