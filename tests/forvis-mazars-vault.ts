import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ForvisMazarsVault } from "../target/types/forvis_mazars_vault";
import { expect } from "chai";

describe("forvis-mazars-vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.forvisMazarsVault as Program<ForvisMazarsVault>;
  const user = provider.wallet.publicKey;

  // Derive PDAs
  const [vaultStatePda, stateBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), user.toBuffer()],
    program.programId
  );

  const [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultStatePda.toBuffer()],
    program.programId
  );

  before(async () => {
    // Airdrop for fees 
    await provider.connection.requestAirdrop(user, 10 * anchor.web3.LAMPORTS_PER_SOL);
    // Wait for confirmation
    await new Promise(resolve => setTimeout(resolve, 1000));
  });

  it("Initialize the vault", async () => {
    let tx = await program.methods
      .initialize()
      .accountsStrict({
        user: user,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("\nInitialize Instruction Successful, Tx Signature: ", tx);

    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    expect(vaultState.vaultBump).to.equal(vaultBump);
    expect(vaultState.stateBump).to.equal(stateBump);

    const vaultBalance = await provider.connection.getBalance(vaultPda);
    const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(0);
    expect(vaultBalance).to.equal(rentExempt);
  });

  it("Deposit SOL into the vault", async () => {
    const depositAmount = 1 * anchor.web3.LAMPORTS_PER_SOL; // 1 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialUserBalance = await provider.connection.getBalance(user);

    let tx = await program.methods
    .deposit(new anchor.BN(depositAmount))
    .accountsStrict({
      user: user,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    const finalUserBalance = await provider.connection.getBalance(user);

    console.log("\nDeposit Instruction Successful, Tx Signature: ", tx);

    console.log("\nInitial User Balance:", initialUserBalance);
    console.log("Final User Balance:", finalUserBalance);

    console.log("\nInitial Vault Balance:", initialVaultBalance);
    console.log("Final Vault Balance:", finalVaultBalance);
  });

  it("Withdraw SOL from the vault", async () => {
    const withdrawAmount = 0.5 * anchor.web3.LAMPORTS_PER_SOL; // 0.5 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialUserBalance = await provider.connection.getBalance(user);

    let tx = await program.methods
      .withdraw(new anchor.BN(withdrawAmount))
      .accountsStrict({
        user: user,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    const finalUserBalance = await provider.connection.getBalance(user);

    console.log("\nWithdraw Instruction Successful, Tx Signature: ", tx);

    console.log("\nInitial User Balance:", initialUserBalance);
    console.log("Final User Balance:", finalUserBalance);

    console.log("\nInitial Vault Balance:", initialVaultBalance);
    console.log("Final Vault Balance:", finalVaultBalance);
  });

  it("Close the vault", async () => {
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialVaultStateBalance = await provider.connection.getBalance(vaultStatePda);
    const initialUserBalance = await provider.connection.getBalance(user);

    let tx = await program.methods
      .close()
      .accountsStrict({
        user: user,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalUserBalance = await provider.connection.getBalance(user);

    console.log("\nClose Instruction Successful, Tx Signature: ", tx);

    // Vault should be 0
    expect(await provider.connection.getBalance(vaultPda)).to.equal(0);

    // VaultState should be closed (null)
    const vaultStateInfo = await provider.connection.getAccountInfo(vaultStatePda);
    expect(vaultStateInfo).to.be.null;

    // User gets back the remaining balance - fees
    expect(finalUserBalance).to.equal(initialUserBalance + initialVaultBalance + initialVaultStateBalance - 5000);
  });
});
