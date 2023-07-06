


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


    step 1) charge server and user wallet
    step 2) init user pda must be called to create user pda on chain, this can be done by server to avoid double signing by user 
    step 3) init match pda by server
    step 4) user sends SOL to user pda 
    step 5) server call deposit method to send SOL from user pda to match pda 
    step 6) start game 
    step 7) at any time user can call withdraw method which transfers SOL from user pda to user wallet 
    step 8) finish game




startGame ( matchID : int ,players : PublicKey[] , serverDeckEncoded : string , betValue : BigInt ) => Contract Stores chainDeck [ 528 byte ] on GamePDA 
finishGame ( matchID , winners : PublicKey[] , serverDeckKey : string , ipfsLink : string ) 



*/



use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("YRDxsg529tECpHUToZ61uMfUeWF2fDCzLeJoLNq4dFt");



#[program]
pub mod ognils {


    use super::*;
    
    pub fn init_match_pda(ctx: Context<InitMatchPda>, match_id: String, bump: u8) -> Result<()>{

        let server = &ctx.accounts.server;
        let match_pda_data = &mut ctx.accounts.match_pda;

        if &ctx.accounts.signer.key() != &server.key(){
            return err!(ErrorCode::RestrictionError);
        }

        match_pda_data.match_id = match_id.clone();
        match_pda_data.bump = bump;
        match_pda_data.server = *server.key;


        emit!(MatchEvent{ 
            match_id, 
            players: match_pda_data.players, 
            server: server.key(), 
            server_commit: match_pda_data.server_commit.clone(), 
            bet_value: match_pda_data.bet_value, 
            bump: match_pda_data.bump, 
            server_key: match_pda_data.server_key.clone(), 
            ipfs_link: match_pda_data.ipfs_link.clone(), 
            chain_table: match_pda_data.chain_table 
        });

        Ok(())

    }
    
    pub fn init_user_pda(ctx: Context<InitUserPda>, match_id: String) -> Result<()> {
        
        let user_pda = &ctx.accounts.user_pda;
        
        //// since there is no data inside user PDA account
        //// there is no need to mutate anything in here,
        //// chill and call next method :)
        
        // chill zone

        //...
        
        Ok(())

    } 

    pub fn withdraw(ctx: Context<WithdrawFromUserPda>, player_bump: u8, match_id: String, amount: u64) -> Result<()>{

        let user_pda = &mut ctx.accounts.user_pda;
        let signer = &ctx.accounts.signer; //// only player can withdraw
        let player = &ctx.accounts.player;
        let player_pda_balance = player.try_lamports()?; 

        if signer.key != player.key{
            return err!(ErrorCode::RestrictionError);
        }

        if player_pda_balance > 0 && player_pda_balance > amount{
            **user_pda.to_account_info().try_borrow_mut_lamports()? -= amount;
            **player.try_borrow_mut_lamports()? += amount;
        } else{
            return err!(ErrorCode::PlayerBalanceIsZero);
        }
    

        Ok(())
        
    }

    pub fn server_withdraw(ctx: Context<WithdrawFromMatchPda>, match_id: String, amount: u64) -> Result<()>{

        let server_pda = &mut ctx.accounts.match_pda;
        let signer = &ctx.accounts.signer; //// only player can withdraw
        let server = &ctx.accounts.server;
        let server_pda_balance = server.try_lamports()?; 

        if signer.key != server.key{
            return err!(ErrorCode::RestrictionError);
        }


        if server_pda_balance > 0 && server_pda_balance > amount {
            **server_pda.to_account_info().try_borrow_mut_lamports()? -= amount;
            **server.try_borrow_mut_lamports()? += amount;
        } else{
            return err!(ErrorCode::PlayerBalanceIsZero);
        }
    

        Ok(())
        
    }

    pub fn deposit(ctx: Context<DepositToMatchPda>, match_id: String, amount: u64) -> Result<()>{

        let user_pda_account = &mut ctx.accounts.user_pda;
        let match_pda = &mut ctx.accounts.match_pda;
        let match_pda_account = match_pda.to_account_info();
        let user_pda_lamports = user_pda_account.to_account_info().lamports();
        let signer = ctx.accounts.signer.key();
        let server = ctx.accounts.server.key();
        let palyer = ctx.accounts.player.key();
        
        // ----------------- finding a PDA logic ----------------- 
        // let program_id = ctx.accounts.system_program.to_account_info();
        // let player_pubkey = user_pda_account.key();
        // let player_seeds = &[b"ognils", player_pubkey.as_ref()]; //// this is of type &[&[u8]; 2]
        // let player_pda = Pubkey::find_program_address(player_seeds, &program_id.key()); //// output is an off curve public key and a bump that specify the iteration that this public key has generated 
        // let player_pda_account = player_pda.0;

        if user_pda_lamports < amount {
            return err!(ErrorCode::InsufficientFund);
        }

        // only server can deposit into match pda account by withdrawing from user pda account
        // means that first user must deposit into his/her pda then server can take the amount 
        // from his/her pda and transfer to the match pda
        if signer != server{
            return err!(ErrorCode::RestrictionError);
        } 

        **user_pda_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **match_pda_account.try_borrow_mut_lamports()? += amount;

        Ok(())

    }

    pub fn start_game(ctx: Context<StartGame>, 
            match_id: String, 
            bump: u8,
            players: [Option<Pubkey>; 6],
            server_commit: String, 
            bet_value: u64,
        ) -> Result<()>
    {

        let server = &ctx.accounts.server;
        let server_pda = &mut ctx.accounts.match_pda;
        let signer = &ctx.accounts.signer;

        if signer.key != server.key{
            return err!(ErrorCode::RestrictionError);
        }


        if **server_pda.to_account_info().try_borrow_lamports()? < bet_value{
            return err!(ErrorCode::PdaCantHaveAmountLowerThanBetValue);
        }


        let mut last_nonce = 0;
        let mut new_table = Vec::<u8>::new();
        for nonce in 1..17{

            let _32bytes_input = format!("{}${}", nonce, server_commit);
            let _32_hash = hash::hash(_32bytes_input.as_bytes());
            let table = &mut _32_hash.try_to_vec().unwrap(); 
            new_table.append(table);

            last_nonce = nonce;

        }

        let last_32bytes_input = format!("{}${}", last_nonce, server_commit);
        let last_hash = hash::hash(last_32bytes_input.as_bytes());
        let last_part_table = &mut last_hash.try_to_vec().unwrap();  
        
        new_table.append(last_part_table);

        let mut chain_table = [0u8; 528];
        for table_idx in 0..528{
            chain_table[table_idx] = new_table[table_idx];
        }
        server_pda.chain_table = chain_table;
        server_pda.match_id = match_id;
        server_pda.players = players;
        server_pda.server_commit = server_commit;
        server_pda.bet_value = bet_value;
        server_pda.bump = bump;



        Ok(())
        
    }

    pub fn finish_game(ctx: Context<FinishGame>, 
            match_id: String,
            server_key: String, 
            ipfs_link: String, 
        ) -> Result<()>{ 
        
        //// since Copy traint is not implemented for ctx.accounts fields
        //// like AccountInfo and Account we must borrow the ctx and because 
        //// AccountInfo and Account fields don't imeplement Copy trait 
        //// we must borrow their instance if we want to move them or 
        //// call a method that takes the ownership of their instance 
        //// like unwrap() in order not to be moved. 

        let match_pda = &ctx.accounts.match_pda.to_account_info();
        let match_pda_data = &ctx.accounts.match_pda;
        let signer = ctx.accounts.signer.key;
        let server = ctx.accounts.server.key;


        if signer != server{
            return err!(ErrorCode::RestrictionError);
        }

        // ----------------- players accounts ----------------------
        //// can't move out of a type if it's behind a shread reference
        //// if there was Some means we have winners
        let first_player_account = &ctx.accounts.first_user_pda;
        let second_player_account = &ctx.accounts.second_user_pda;
        let third_player_account = &ctx.accounts.third_user_pda;
        let fourth_player_account = &ctx.accounts.fourth_user_pda;
        let fifth_player_account = &ctx.accounts.fifth_user_pda;
        let sixth_player_account = &ctx.accounts.sixth_user_pda;
        let winners = vec![first_player_account, second_player_account, third_player_account,
                                                     fourth_player_account, fifth_player_account, sixth_player_account];
        
        {
            let mut winner_count = 0;
            let current_match_pda_amout = **ctx.accounts.match_pda.to_account_info().try_borrow_lamports()?;
            if current_match_pda_amout > 0{

                let winner_flags = winners
                    .clone()
                    .into_iter()
                    .map(|w|{
                        if w.is_some(){
                            winner_count += 1;
                            true
                        } else{
                            false
                        }
                    })
                    .collect::<Vec<bool>>();
                
                let winner_reward = match_pda_data.bet_value / winner_count; //// spread between winners equally
                let ramining_in_pda = current_match_pda_amout - winner_reward;
                

                for is_winner_idx in 0..winner_flags.len(){
                    //// every element inside winner_flags is a boolean map to the winner index inside the winners 
                    //// vector also player accounts are behind a shared reference thus we can't move out of them
                    //// since unwrap(self) method takes the ownership of the type and return the Self because 
                    //// in its first param doesn't borrow the self or have &self, the solution is to use a borrow 
                    //// of the player account then unwrap() the borrow type like first_player_account.as_ref().unwrap()
                    //// with this way we don't lose the ownership of the first_player_account and we can call 
                    //// the to_account_info() method on it.
                    if winner_flags[is_winner_idx]{
                        let winner = winners[is_winner_idx].clone();
                        let winner_account = winner.as_ref().unwrap().to_account_info();
                        **winner_account.try_borrow_mut_lamports()? += winner_reward;
                        **match_pda.try_borrow_mut_lamports()? -= winner_reward;
                    }
                }

            } else{
                return err!(ErrorCode::MatchPdaIsEmpty);
            }
        }

        /* can't have first a mutable borrow then immutable one we must make sure that we have only one mutable borrow in each scope */
        let mut match_pda_data = &mut ctx.accounts.match_pda;
        match_pda_data.server_key = server_key;
        match_pda_data.ipfs_link = ipfs_link;

        emit!(MatchEvent{ 
            match_id, 
            players: match_pda_data.players, 
            server: *server, 
            server_commit: match_pda_data.server_commit.clone(), 
            bet_value: match_pda_data.bet_value, 
            bump: match_pda_data.bump, 
            server_key: match_pda_data.server_key.clone(), 
            ipfs_link: match_pda_data.ipfs_link.clone(), 
            chain_table: match_pda_data.chain_table 
        });

        Ok(())

    }

}



#[account]
pub struct MatchPda{
    pub match_id: String, 
    pub players: [Option<Pubkey>; 6],
    pub server: Pubkey,
    pub server_commit: String, 
    pub bet_value: u64,
    pub bump: u8,
    pub server_key: String,
    pub ipfs_link: String,
    pub chain_table: [u8; 528]
}

#[account]
pub struct UserPda{
    user_wallet: Pubkey,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct InitUserPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// player
   /// CHECK:
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   /// CHECK:
   #[account(init, payer = signer, space = 300, seeds = [b"ognils", player.key().as_ref()], bump)]
   pub user_pda: Account<'info, UserPda>,
   #[account(mut, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct DepositToMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   /// CHECK:
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   /// CHECK:
   #[account(mut, seeds = [b"ognils", player.key().as_ref()], bump)]
   pub user_pda: Account<'info, UserPda>,
   #[account(mut, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,
   pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct InitMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   /// CHECK:
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   #[account(init, payer = signer, space = 8192, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String, bump: u8)]
pub struct StartGame<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// only server
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   #[account(mut, seeds = [match_id.as_bytes(), 
                            match_pda.server.key().as_ref()], 
                            bump = bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(player_bump: u8, match_id: String)]
pub struct WithdrawFromUserPda<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only player
   /// CHECK:
    #[account(mut, seeds = [b"ognils", player.key().as_ref()], bump = player_bump)]
    pub user_pda: Account<'info, UserPda>,
    #[account(mut, seeds = [match_id.as_bytes(), 
                            match_pda.server.key().as_ref()], 
                            bump = match_pda.bump)]
    pub match_pda: Box<Account<'info, MatchPda>>,
    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct WithdrawFromMatchPda<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only server
    #[account(mut, seeds = [match_id.as_bytes(), 
                            match_pda.server.key().as_ref()], 
                            bump = match_pda.bump)]
    pub match_pda: Box<Account<'info, MatchPda>>,
    /// CHECK:
    #[account(mut)]
    pub server: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct FinishGame<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only server
   /// CHECK:
   #[account(mut, seeds = [match_id.as_bytes(), 
                            match_pda.server.key().as_ref()], 
                            bump = match_pda.bump)]
    pub match_pda: Box<Account<'info, MatchPda>>,
    /// CHECK:
    #[account(mut)]
    pub server: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub first_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub second_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub third_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub fourth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub fifth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut)]
    pub sixth_user_pda: Option<AccountInfo<'info>>,
    pub system_program: Program<'info, System>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Restriction error!")]
    RestrictionError,
    #[msg("Player Doesn't Exist!")]
    PlayerDoesntExist,
    #[msg("Player Balance Is Zero!")]
    PlayerBalanceIsZero,
    #[msg("Match Is Locked!")]
    MatchIsLocked,
    #[msg("Match PDA Is Empty!")]
    MatchPdaIsEmpty,
    #[msg("Match PDA Can't Have Amount Lower Than Bet Value")]
    PdaCantHaveAmountLowerThanBetValue
}


#[event]
#[derive(Debug)]
pub struct MatchEvent{
    pub match_id: String, 
    pub players: [Option<Pubkey>; 6],
    pub server: Pubkey,
    pub server_commit: String, 
    pub bet_value: u64,
    pub bump: u8,
    pub server_key: String,
    pub ipfs_link: String,
    pub chain_table: [u8; 528]
}
