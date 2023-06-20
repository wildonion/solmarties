import * as anchor from "@project-serum/anchor";
import { Program, BorshCoder, EventParser } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { Ognils } from "../target/types/ognils";
import { assert, expect } from "chai";



/*

    for every player `init_user_pda` must be called before starting the game 
    to initialize all the player PDAs inside the queue (mmq) for the current match

    `init_match_pda` needs to be called by the server or a higher authority
    to initialize the match PDA account and initialize its first data on chain.

    `deposit` and `withdraw` both can be called by the user to deposit into 
    the match PDA and withdraw from the user PDA account.

    `start_game` will be called by the server after initializing the PDAs 
    to generate the game logic on chain thus all player public keys inside 
    the queue (mmq) must be passed into this call. 

    `finish_game` must be called by the server after the game has finished 
    to pay the winners, thus it requires all the player PDAs to be passed 
    in to the call, also there must be 6 PDAs inside the call since maximum
    players inside the queue are 6 thus not all of them can be Some, it must 
    be checked for its Some part before paying the winner.  

*/



describe("ognils", () => {

  // TODO - use a real provider or connection like testnet or devnet
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  



  const player = anchor.web3.Keypair.generate(); // TODO - wallet handler
  const server = anchor.web3.Keypair.generate(); // TODO - server public key
  const revenue_share_wallet = anchor.web3.Keypair.generate(); // TODO - staking pool account
  


  const lamport_amount = 10_000_000_000;
  const bet_amount = 1_000_000_000;
  const reserve_amount = 5_000_000_000; //// the amount of ticket




  // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
  const program = anchor.workspace.Ognils as Program<Ognils>;
  const provider = anchor.AnchorProvider.env(); 

  
  
  it("PDAs created!", async () => {


    // find pda account for game account
    const [gameStatePDA, bump] = PublicKey
    .findProgramAddressSync(
        [server.publicKey.toBuffer(), player.publicKey.toBuffer()],
        program.programId
      )


      // find pda for the ticket reservation account
    const [ticketStatsPDA, _bump] = PublicKey
    .findProgramAddressSync(
        [server.publicKey.toBuffer(), player.publicKey.toBuffer()],
        program.programId
      )



      ///////////////////////////////
      /////// STEP 0
      ///////////////////////////////

      //----------------------------
      // player one charging account
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





  
  });
});
