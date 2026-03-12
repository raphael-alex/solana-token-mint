import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StmTokenMintProgram } from "../target/types/stm_token_mint_program";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { MerkleTree } from "merkletreejs";
import keccak256 from "keccak256";
import { Buffer } from "buffer";

const CONFIG_SEEDS = [Buffer.from("stm_config")];
const MINT_SEEDS = [Buffer.from("stm_mint")];
const VAULT_SEEDS = [Buffer.from("stm_vault")];
const AUTHORITY_SEEDS = [Buffer.from("vault_authority")];

let configPda: PublicKey;
let mintPda: PublicKey;
let vaultPda: PublicKey;
let vaultAuthorityPda: PublicKey;
let stakePda: PublicKey;
let airdropStatusPda: PublicKey;
let claimStatusPda: PublicKey;

interface AirdropItem {
  address: anchor.web3.PublicKey;
  campaign_id: string;
  amount: string;
}

describe("solana-token-mint", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const project_wallet = provider.wallet as anchor.Wallet;
  console.log("Wallet public key:", project_wallet.publicKey.toBase58());

  const TOKEN_PROGRAM_ID = anchor.utils.token.TOKEN_PROGRAM_ID;

  const user_A = new Keypair();
  const user_B = new Keypair();
  const user_C = new Keypair();
  const program = anchor.workspace
    .stmTokenMintProgram as Program<StmTokenMintProgram>;
  const campaign_id = new anchor.BN(1);

  const airdropList: AirdropItem[] = [
    {
      address: user_A.publicKey,
      campaign_id: campaign_id.toString(),
      amount: "1000",
    },
    {
      address: user_B.publicKey,
      campaign_id: campaign_id.toString(),
      amount: "2000",
    },
    {
      address: user_C.publicKey,
      campaign_id: campaign_id.toString(),
      amount: "3000",
    },
  ];

  const leaves = airdropList.map((item) => {
    const amountBuffer = Buffer.alloc(8);
    amountBuffer.writeBigUInt64LE(BigInt(item.amount));

    const campaignIdBuffer = Buffer.alloc(8);
    campaignIdBuffer.writeBigUInt64LE(BigInt(item.campaign_id));
    return keccak256(
      Buffer.concat([
        item.address.toBuffer(),
        Buffer.from(campaignIdBuffer),
        Buffer.from(amountBuffer),
      ])
    );
  });

  const tree = new MerkleTree(leaves, keccak256, { sortPairs: true });
  const root_buffer = tree.getRoot();
  const root = Array.from(root_buffer);

  const leafToProve = leaves[1];
  const hexProof = tree.getHexProof(leafToProve);
  console.log(`Proof for AddressA:`, hexProof);
  const proof = hexProof.map((p) =>
    Array.from(Buffer.from(p.replace(/^0x/, ""), "hex"))
  );

  // --- PDA ADDRESSES (计算一次，所有测试使用) ---
  before(async () => {
    const STAKE_SEEDS = [
      Buffer.from("staking_account"),
      user_A.publicKey.toBuffer(),
    ];
    // const CLAIM_STATUS_SEEDS = [
    //   Buffer.from("claim_status"),
    //   campaign_id.toBuffer("le", 8),
    //   user_A.publicKey.toBuffer(),
    // ];
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      CONFIG_SEEDS,
      program.programId
    );
    [mintPda] = anchor.web3.PublicKey.findProgramAddressSync(
      MINT_SEEDS,
      program.programId
    );
    [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      VAULT_SEEDS,
      program.programId
    );
    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      AUTHORITY_SEEDS,
      program.programId
    );
    [stakePda] = anchor.web3.PublicKey.findProgramAddressSync(
      STAKE_SEEDS,
      program.programId
    );
    // [claimStatusPda] = anchor.web3.PublicKey.findProgramAddressSync(
    //   CLAIM_STATUS_SEEDS,
    //   program.programId
    // );
    console.log(`Config PDA: ${configPda.toBase58()}`);
    // console.log(`claim status PDA: ${claimStatusPda.toBase58()}`);
    console.log(
      "campaign_id buffer (hex):",
      campaign_id.toBuffer("le", 8).toString("hex")
    );
  });

  it("Is initialize!", async () => {
    console.log("APY:", 50_000_000);
    const tx = await program.methods
      .initialize(project_wallet.publicKey, new anchor.BN(50_000_000))
      .accounts({
        admin: project_wallet.publicKey,
      })
      .rpc();
    console.log("Your [Initialize] transaction signature", tx);
  });

  it("Is set pause!", async () => {
    console.log("Set pause");
    const tx = await program.methods
      .setPause(false)
      .accounts({
        admin: project_wallet.publicKey,
      })
      .rpc();
    console.log("Your [SetPause] transaction signature", tx);
  });

  it("Is create and mint tokens!", async () => {
    let wallet_balance = await provider.connection.getBalance(
      project_wallet.publicKey
    );
    console.log("Project wallet balance:", wallet_balance);
    const amount = new anchor.BN(4_000_000_000);
    const tx = await program.methods
      .createAndMintTokens(amount)
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Your [Create And Mint Tokens] transaction signature", tx);
    wallet_balance = await provider.connection.getBalance(
      project_wallet.publicKey
    );
    console.log("Project wallet balance:", wallet_balance);
    const vault_balance = await provider.connection.getTokenAccountBalance(
      vaultPda
    );
    console.log("vault balance:", vault_balance.value.uiAmountString);
  });

  it("Is increase issuance!", async () => {
    const amount = new anchor.BN(4_000_000_000);
    const tx = await program.methods
      .increaseIssuance(amount)
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Your [Increase Issuance] transaction signature", tx);
    const vault_balance = await provider.connection.getTokenAccountBalance(
      vaultPda
    );
    console.log("vault balance:", vault_balance.value.uiAmountString);
  });

  it("Is init airdrop!", async () => {
    console.log("APY:", 50_000_000);
    const nowInSecond = Math.floor(Date.now() / 1000);
    const tx = await program.methods
      .initAirdrop(
        campaign_id,
        new anchor.BN(1_000_000_000_000_000),
        root,
        new anchor.BN(nowInSecond - 60),
        new anchor.BN(nowInSecond + 1000000)
      )
      .accounts({
        // admin: project_wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Your [init airdrop campaign] transaction signature", tx);
  });

  // it("Is emergency withdraw", async () => {
  //   console.log("Emergency withdraw");
  //   const tx = await program.methods
  //     .withdrawEmergency(campaign_id)
  //     .accounts({
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .rpc();
  //   console.log("Your [Emergency withdraw] transaction signature", tx);
  // });

  it("Is withdraw !", async () => {
    const AIRDROP_STATUS_SEEDS = [
      Buffer.from("airdrop_info"),
      campaign_id.toBuffer("le", 8),
    ];
    [airdropStatusPda] = anchor.web3.PublicKey.findProgramAddressSync(
      AIRDROP_STATUS_SEEDS,
      program.programId
    );
    let configAirdropStatus = await program.account.airdropInfo.fetch(
      airdropStatusPda
    );
    console.log(
      "airdrop_info capacity:",
      configAirdropStatus.capacity.toString()
    );
    let recipient_ata_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );

    console.log("Payer ATA balance:", recipient_ata_balance);
    const amount = new anchor.BN(5_000_000_000_000);
    const tx = await program.methods
      .withdrawTokens(campaign_id, amount)
      .accounts({
        recipient: user_A.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Your [withdraw Tokens] transaction signature", tx);
    recipient_ata_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );

    console.log("Payer ATA balance:", recipient_ata_balance);
    configAirdropStatus = await program.account.airdropInfo.fetch(
      airdropStatusPda
    );
    console.log(
      "airdrop_vault capacity after withdraw:",
      configAirdropStatus.capacity.toString()
    );
  });

  it("Is claim airdrop", async () => {
    const AIRDROP_STATUS_SEEDS = [
      Buffer.from("airdrop_info"),
      campaign_id.toBuffer("le", 8),
    ];
    [airdropStatusPda] = anchor.web3.PublicKey.findProgramAddressSync(
      AIRDROP_STATUS_SEEDS,
      program.programId
    );
    let configAirdropStatus = await program.account.airdropInfo.fetch(
      airdropStatusPda
    );
    console.log(
      "airdrop_info capacity:",
      configAirdropStatus.capacity.toString()
    );
    await airdrop(provider.connection, user_B.publicKey);
    let user_B_balance = await getAtaBalance(
      provider.connection,
      user_B.publicKey,
      mintPda
    );
    console.log("user_B ATA balance(before):", user_B_balance);
    const amount = new anchor.BN(2_000);
    const tx = await program.methods
      .claimAridrop(campaign_id, amount, proof)
      .accounts({
        signer: user_B.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_B])
      .rpc();
    console.log("Your [claim airdrop] transaction signature", tx);
    user_B_balance = await getAtaBalance(
      provider.connection,
      user_B.publicKey,
      mintPda
    );
    console.log("user_B ATA balance(after):", user_B_balance);
    configAirdropStatus = await program.account.airdropInfo.fetch(
      airdropStatusPda
    );
    console.log(
      "airdrop_vault capacity after withdraw:",
      configAirdropStatus.capacity.toString()
    );
  });

  it("Is transfer", async () => {
    let user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(before):", user_A_balance);
    let user_B_balance = await getAtaBalance(
      provider.connection,
      user_B.publicKey,
      mintPda
    );
    console.log("user_B ATA balance(before):", user_B_balance);
    const amount = new anchor.BN(1_000_000_000);
    const tx = await program.methods
      .transferToken(amount)
      .accounts({
        signer: user_A.publicKey,
        feePayer: project_wallet.publicKey,
        recipient: user_B.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_A])
      .rpc();
    console.log("Your [Transfer Tokens] transaction signature", tx);
    user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(after):", user_A_balance);
    user_B_balance = await getAtaBalance(
      provider.connection,
      user_B.publicKey,
      mintPda
    );
    console.log("user_B ATA balance(after):", user_B_balance);
  });

  it("Is stake", async () => {
    let vault_balance = await provider.connection.getTokenAccountBalance(
      vaultPda
    );
    console.log(
      "vault balance(before stake):",
      vault_balance.value.uiAmountString
    );
    await airdrop(provider.connection, user_A.publicKey);
    let user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(before stake):", user_A_balance);
    const amount = new anchor.BN(2_000_000_000_000);
    let tx = await program.methods
      .stake(amount, true)
      .accounts({
        signer: user_A.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_A])
      .rpc();
    console.log("Your [Stake Tokens] transaction signature", tx);
    let configStakingData = await program.account.stakingAccount.fetch(
      stakePda
    );
    console.log("staking account:", configStakingData);
    console.log("Stake Amount(Lamports):", configStakingData.amount.toString());
    console.log("Stake bump:", configStakingData.bump);
    console.log(
      "Stake lastStakeTime:",
      configStakingData.claimableReward.toString()
    );
    console.log("Stake owner:", configStakingData.owner);
    console.log("Stake URD:", configStakingData.userRewardDebt.toString());

    let configData = await program.account.configuration.fetch(configPda);
    console.log(
      "GRPC in Configuration:",
      configData.globalRewardPerShare.toString()
    );
    console.log(
      "total stake amount in Configuration:",
      configData.totalStakedAmount.toString()
    );

    await new Promise((resolve) => setTimeout(resolve, 2000));
    tx = await program.methods
      .stake(amount, true)
      .accounts({
        signer: user_A.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_A])
      .rpc();
    console.log("Your [Stake Tokens Twice] transaction signature", tx);
    user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(after stake):", user_A_balance);

    vault_balance = await provider.connection.getTokenAccountBalance(vaultPda);
    console.log(
      "vault balance(after stake):",
      vault_balance.value.uiAmountString
    );

    configStakingData = await program.account.stakingAccount.fetch(stakePda);
    console.log("Stake Amount(Lamports):", configStakingData.amount.toString());
    console.log("Stake bump:", configStakingData.bump);
    console.log(
      "Stake lastStakeTime:",
      configStakingData.claimableReward.toString()
    );
    console.log("Stake owner:", configStakingData.owner);
    console.log("Stake URD:", configStakingData.userRewardDebt.toString());

    configData = await program.account.configuration.fetch(configPda);
    console.log(
      "GRPC in Configuration:",
      configData.globalRewardPerShare.toString()
    );
    console.log(
      "total stake amount in Configuration:",
      configData.totalStakedAmount.toString()
    );
  });

  //   it("Is claim rewards", async () => {
  //     let user_A_balance = await getAtaBalance(
  //       provider.connection,
  //       user_A.publicKey,
  //       mintPda
  //     );
  //     console.log("user_A ATA balance(before):", user_A_balance);
  //     await new Promise((resolve) => setTimeout(resolve, 9000));
  //     const tx = await program.methods
  //       .claimReward()
  //       .accounts({
  //         signer: user_A.publicKey,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .signers([user_A])
  //       .rpc();
  //     console.log("Your [claim rewards] transaction signature", tx);
  //     user_A_balance = await getAtaBalance(
  //       provider.connection,
  //       user_A.publicKey,
  //       mintPda
  //     );
  //     console.log("user_A ATA balance(after):", user_A_balance);
  //   });
  //
  it("Is unstake", async () => {
    console.log("Unstake");
    let vault_balance = await provider.connection.getTokenAccountBalance(
      vaultPda
    );
    console.log(
      "vault balance(before unstake):",
      vault_balance.value.uiAmountString
    );
    let user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(before unstake):", user_A_balance);
    // TODO
    const amount = new anchor.BN(5_000_000_000);
    const tx = await program.methods
      .unstake(amount, false)
      .accounts({
        signer: user_A.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user_A])
      .rpc();
    console.log("Your [unstake Tokens] transaction signature", tx);

    user_A_balance = await getAtaBalance(
      provider.connection,
      user_A.publicKey,
      mintPda
    );
    console.log("user_A ATA balance(after unstake):", user_A_balance);

    vault_balance = await provider.connection.getTokenAccountBalance(vaultPda);
    console.log(
      "vault balance(after unstake):",
      vault_balance.value.uiAmountString
    );

    const configStakingData = await program.account.stakingAccount.fetch(
      stakePda
    );
    console.log("Stake Amount(Lamports):", configStakingData.amount.toString());
    console.log("Stake bump:", configStakingData.bump);
    console.log(
      "Stake claimableReward:",
      configStakingData.claimableReward.toString()
    );
    console.log("Stake owner:", configStakingData.owner);
    console.log(
      "Stake user reward debt:",
      configStakingData.userRewardDebt.toString()
    );
  });

  it("Is burn", async () => {
    console.log("Burn");
    let vault_balance = await provider.connection.getTokenAccountBalance(
      vaultPda
    );
    console.log(
      "vault balance(before unburn):",
      vault_balance.value.uiAmountString
    );

    const tx = await program.methods
      .burnTokens(new anchor.BN(1_000_000_000))
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    vault_balance = await provider.connection.getTokenAccountBalance(vaultPda);
    console.log(
      "vault balance(after burn):",
      vault_balance.value.uiAmountString,
      tx
    );
  });
});

async function getAtaBalance(
  connection: Connection,
  ownerPublicKey: PublicKey,
  mintPublicKey: PublicKey
): Promise<string> {
  try {
    const ataPublicKey = getAssociatedTokenAddressSync(
      mintPublicKey,
      ownerPublicKey,
      false
    );
    console.log("ATA Public Key:", ataPublicKey.toBase58());

    const balanceAccount = await connection.getTokenAccountBalance(
      ataPublicKey
    );

    if (balanceAccount.value.uiAmount === null) {
      return "0";
    }

    return balanceAccount.value.uiAmountString;
  } catch (error) {
    console.error("Error getting ATA balance:", error);
    return "0";
  }
}

async function airdrop(connection: Connection, user: PublicKey) {
  await connection.requestAirdrop(user, anchor.web3.LAMPORTS_PER_SOL * 3);

  await new Promise((resolve) => setTimeout(resolve, 1000));
}
