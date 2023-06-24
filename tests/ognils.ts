import * as anchor from "@project-serum/anchor";
import { Program, BorshCoder, EventParser } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { Ognils } from "../target/types/ognils";
import { assert, expect } from "chai";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";



/*

    for every player `init_user_pda` must be called before starting the game 
    to initialize all the player PDAs inside the queue (mmq) for the current match

    `init_match_pda` needs to be called by the server or a higher authority
    to initialize the match PDA account and initialize its first data on chain.

    `deposit` can be called by server and `withdraw` can be called by the user 
    to deposit into the match PDA and withdraw from the user PDA account.

    `start_game` will be called by the server after initializing the PDAs 
    to generate the game logic on chain thus all player public keys inside 
    the queue (mmq) must be passed into this call. 

    `finish_game` must be called by the server after the game has finished 
    to pay the winners, thus it requires all the player PDAs to be passed 
    in to the call, also there must be 6 PDAs inside the call since maximum
    players inside the queue are 6 thus not all of them can be Some, it must 
    be checked for its Some part before paying the winner.  

*/



describe("Ognils", () => {

  // TODO - use a real provider or connection like testnet or devnet
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const server = anchor.web3.Keypair.generate(); // TODO - server public key
  const lamport_amount = 10_000_000_000;
  const bet_amount = 1_000_000_000;
  // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
  const program = anchor.workspace.Ognils as Program<Ognils>;
  const provider = anchor.AnchorProvider.env(); 
  const match_id = "match-id"; /////// TODO -



  //////// TODO - maximum players can be 6 inside mmq
  const player = anchor.web3.Keypair.generate(); // TODO - wallet handler
  interface PlayerInfo{
    pubKey: string;
    commit: string;
  };
  let info1: PlayerInfo = {
    pubKey: player.publicKey.toString(),
    commit: "player-random-seed-commit"
  };
  
   
  
  
  it("Game Started!", async () => {

      // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--=-=-=
      // -=-=--=--=--=--=--=--=--=- CHARGING OPS -=-=--=--=--=--=--=--=--=-
      // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--=-=-=

      //----------------------------
      // player charging account
      //----------------------------
      const latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction ({
        blockhash: latestBlockHashforUserOne.blockhash,
        lastValidBlockHeight: latestBlockHashforUserOne.lastValidBlockHeight,
        signature: await provider.connection.requestAirdrop(player.publicKey, lamport_amount)
      });
      console.log("player balance: ", await provider.connection.getBalance(player.publicKey));

      //----------------------------
      // server charging account
      //----------------------------
      const _latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction ({
        blockhash: latestBlockHashforUserOne.blockhash,
        lastValidBlockHeight: _latestBlockHashforUserOne.lastValidBlockHeight,
        signature: await provider.connection.requestAirdrop(server.publicKey, lamport_amount)
      });
      console.log("server balance: ", await provider.connection.getBalance(server.publicKey));


      // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--
      // -=-=--=--=--=--=--=--=--=- PDA OPS -=-=--=--=--=--=--=--=--=-
      // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--

      // find match pda account for game
      const [matchPDA, match_pda_bump] = PublicKey
      .findProgramAddressSync(
          [Buffer.from(match_id, "utf-8"), server.publicKey.toBuffer()],
          program.programId
        )
      
      // init match pda
      await program.methods.initMatchPda(match_id, match_pda_bump)
        .accounts({signer: server.publicKey, player: player.publicKey, server: server.publicKey, matchPda: matchPDA
          }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server

      // find pda account for player
      const [userPDA, user_pda_bump] = PublicKey
      .findProgramAddressSync(
          [Buffer.from("slingo", "utf-8"), player.publicKey.toBuffer()],
          program.programId
        )

      // init user pda
      await program.methods.initUserPda()
      .accounts({signer: server.publicKey, player: player.publicKey, server: server.publicKey, userPda: userPDA, matchPda: matchPDA
        }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
      
      //---------------------------------
      // sending from player to user PDA
      //---------------------------------
      // NOTE - at any time player can deposit into his/her pda 
      let tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
        fromPubkey: player.publicKey,
        toPubkey: userPDA,
        lamports: bet_amount,    
      }));
      await anchor.web3.sendAndConfirmTransaction(provider.connection, tx_data, [player]);

      //----------------------------------
      // sending from server to match PDA
      //----------------------------------
      // only server can call the deposit method to 
      // transfer from the user pda into match pda 
      // but the deposited amount passed into the call
      // must be equals to the one that player is 
      // deposited before  
      await program.methods.deposit(new anchor.BN(1_000_000_000))
        .accounts({signer: server.publicKey, player: player.publicKey, server: server.publicKey, userPda: userPDA, matchPda: matchPDA
          }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
        
      //----------------------------------
      // withdraw from match pda by player
      //----------------------------------
      await program.methods.withdraw(user_pda_bump, new anchor.BN(1_000_000_000))
      .accounts({signer: player.publicKey, player: player.publicKey,  userPda: userPDA, matchPda: matchPDA
        }).signers([player]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
    
          

    // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--=
    // -=-=--=--=--=--=--=--=--=- GAME OPS -=-=--=--=--=--=--=--=--=-
    // -=-=--=--=--=--=--=--=--=--=-=--=--=--=--=--=--=--=--=-=--=--=
    

    let announce_commit = "announce-random-seed-commit"; /////// TODO - 
    let players = [info1]; /////// TODO - 
    let rounds = 10; /////// TODO - 
    let size: 5; /////// TODO - 

    //----------------------
    // start game
    //----------------------
    await program.methods.startGame(
          players,
          match_pda_bump,
          rounds,
          size,
          match_id,
          announce_commit,
        )
        .accounts({
            signer: server.publicKey, 
            server: server.publicKey, 
            matchPda: matchPDA
          }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
      
    
    //----------------------
    // finish game
    //----------------------
    await program.methods.finishGame([user_pda_bump])
    .accounts({
        signer: server.publicKey, 
        server: server.publicKey, 
        matchPda: matchPDA,
        firstUserPda: userPDA,
        secondUserPda: null,
        thirdUserPda: null,
        fourthUserPda: null,
        fifthUserPda: null,
        sixthUserPda: null,
      }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
  


  
  });
});
