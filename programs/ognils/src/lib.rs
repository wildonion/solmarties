


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




use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("3CMhnATUZGWaS91YJuQNBzdvBy8fGRprCFxTyMzGkZ45");



/* 
    every two chars in hex is 1 byte since every 4 bits is a hex char
    thus (0xFFFFFFFF) is 4 bytes long or 8 chars in hex
*/
pub const VALUE_JOKER: i32 = -1; 

pub fn generate_cell_values_for_player(player_commit: String) -> Vec<u8>{

    let input = format!("{}", player_commit); //// sha256 bits has 32 bytes length, each of which is in a range between 0 up to 255 
    let hash = hash::hash(input.as_bytes());
    let cells = hash.try_to_vec().unwrap();
    cells
}

pub fn generate_announce_values(annouce_commit: String) -> Vec<u8>{

    let input = format!("{}", annouce_commit); //// sha256 bits has 32 bytes length, each of which is in a range between 0 up to 255 
    let hash = hash::hash(input.as_bytes());
    let announ_vals = hash.try_to_vec().unwrap();
    announ_vals
}


pub fn create_table(size: i32, player_commit: String) -> Vec<Cell>{

    let cell_random_values = crate::generate_cell_values_for_player(player_commit);
    let mut table = vec![];

    fn is_duplicate_fn(val: i32, col_vals: &mut Vec<i32>) -> bool{
        for av_idx in 0..col_vals.len(){
            if col_vals[av_idx] == val{
                return true;
            } else{
                return false;
            }
        }
        return false;
    }

    for row_idx in 0..size{
        
        let mut col_vals: Vec<i32> = vec![];
        let mutable_pointer_to_col_vals = &mut col_vals;
        let (min, max) = get_column_range(row_idx);
        for val in cell_random_values.clone(){
            if min <= val as i32 && val as i32 <= max{
                if is_duplicate_fn(val as i32, mutable_pointer_to_col_vals){
                    continue;
                } else{
                    /*
                        we can't have a mutable borrow more that once in a scope
                        thus we must mutate the actual typa which is col_vals 
                        using its mutable pointer because there is another mutable
                        borrow which is passed to the is_duplicate_fn function
                    */
                    mutable_pointer_to_col_vals.push(val as i32);
                }
            } else{
                continue;
            }
        }

        for col_idx in 0..col_vals.len(){
            for j in 0..cell_random_values.len() - col_idx - 1{
                if col_vals[j] > col_vals[j+1]{
                    (col_vals[j], col_vals[j+1]) = (col_vals[j+1], col_vals[j])  
                }
            }

            table.push(Cell{
                x: row_idx,
                y: col_idx as i32,
                value: col_vals[col_idx]
            });
        }

    }

    table

}

pub fn get_column_range(x: i32) -> (i32, i32){
    let min = x * 20;
    let max = (x + 1) * 20; 
    return (min, max)
}

pub fn create_announced_values(size: i32, max_rounds: i32, annouce_commit: String) -> Vec<Round>{
    
    struct JockerXs{
        pub round_idx: i32,
        pub round_val: i32
    }
    let annouce_random_vals = generate_announce_values(annouce_commit);
    let mut joker_xs: Vec<JockerXs> = vec![];
    let mut round = 0;
    let mut result_announced_values: Vec<Round> = vec![];
    
    /* 
        closures can capture env vars so we can access them inside the closure method, with 
        function we can't do that, since functions have their own scopes, we could either pass 
        the type by value if we don't need its ownership (specially for heap data) or reference 
        if we don't want to lose its ownership inside the caller scope of the method also to mutate 
        the content of the type inside the function without mutating the actual type we 
        must pass a mutable reference to it like for mutating announced_values we must pass 
        the mutable reference to announced_values type to the is_duplicate_fn function, 
        since by mutating the mutable pointer of the main type the actual type will be mutated too, 
    */

    fn is_duplicate_fn(val: i32, val_idx: i32, result_announced_values: &mut Vec<Round>) -> bool{
        for av_idx in 0..result_announced_values.len(){
            if (result_announced_values[av_idx].values[val_idx as usize]) as i32 == val{
                return true;
            } else{
                return false;
            }
        }
        return false;
    }

    /*
        the following closure will borrow and capture the result_announced_values var
        as immutable, thus we can't push into the result_announced_values vector later
        on if we're going to use this method, since rust doesn't allow to borrow the 
        type as mutable if it's borrowed as immutable already in a scope, instead we 
        can use FnMut closure to capture vars mutablyو also announced_values must be 
        initialized in order the closure to be able to capture it into its env
    */
    let is_duplicate = |val: i32, val_idx: i32|{
        for av_idx in 0..result_announced_values.len(){
            if (result_announced_values[av_idx].values[val_idx as usize]) as i32 == val{
                return true;
            } else{
                return false;
            }
        }
        return false;
    };

    for i in (3..max_rounds).step_by(3){
        for announ_val in annouce_random_vals.clone(){
            if 0 <= announ_val as i32 && announ_val as i32 <= size{
                joker_xs.push(JockerXs{
                    round_idx: i,
                    round_val: announ_val as i32,
                }); 
            } else{
                continue;
            }
        }   
    }

    while round < max_rounds{
        let mut announ_vals = vec![0i32];
        for x in 0..size{
            let (min, max) = get_column_range(x);
            for announ_val in annouce_random_vals.clone(){
                let mut av = announ_val as i32;
                if min <= announ_val as i32 && announ_val as i32 <= max{
                    if is_duplicate_fn(announ_val as i32, x, &mut result_announced_values.clone()){
                        continue;
                    } else{
                        for jx in &joker_xs{ /* since Clone trait is not implemented for the JockerXs we must borrow the instance in each iteration to prevent from moving  */
                            if jx.round_idx == round{
                                let jx_rv = jx.round_val;
                                if jx_rv == x{
                                    av = VALUE_JOKER;
                                }
                            } else{
                                continue;
                            }
                        }
                        announ_vals.push(av)
                    }
                }
            }
        }

        result_announced_values.push(
            Round{
                round_val: round,
                values: announ_vals.clone()
            }
        );

        round+=1;
    }

    result_announced_values

}    



#[program]
pub mod ognils {


    use super::*;
    
    pub fn init_match_pda(ctx: Context<InitMatchPda>, match_id: String, bump: u8) -> Result<()>{

        let server = &ctx.accounts.server;
        let match_pda = &mut ctx.accounts.match_pda;

        if &ctx.accounts.signer.key() != &server.key(){
            return err!(ErrorCode::RestrictionError);
        }
        
        match_pda.current_match = CurrentMatch{
            match_id: match_id.clone(),
            bump,
            is_locked: false,
            server: server.key(),
            announced_values: vec![],
            players: vec![],
        };

        msg!("{:#?}", CurrentMatchEvent{ 
            match_id: match_id.clone(), 
            server: server.key(), 
            is_locked: false, 
            announced_values: vec![], 
            players: vec![] 
        });

        emit!(CurrentMatchEvent{ 
            match_id: match_id.clone(), 
            server: server.key(), 
            is_locked: false, 
            announced_values: vec![], 
            players: vec![] 
        });

        Ok(())

    }
    
    pub fn init_user_pda(ctx: Context<InitUserPda>) -> Result<()> {
        
        let user_pda = &ctx.accounts.user_pda;
        
        //// since there is no data inside user PDA account
        //// there is no need to mutate anything in here,
        //// chill and call next method :)
        
        // chill zone

        //...
        
        Ok(())

    } 

    pub fn withdraw(ctx: Context<WithdrawFromUserPda>, player_bump: u8, amount: u64) -> Result<()>{

        let user_pda = &mut ctx.accounts.user_pda;
        let signer = &ctx.accounts.signer; //// only player can withdraw
        let player = &ctx.accounts.player;
        //// accounts fields doesn't implement Copy trait 
        //// like Account fields are not Copy thus we must 
        //// borrow the ctx in order not to move 
        let match_pda = &mut ctx.accounts.match_pda; 
        let match_pda_account = match_pda.to_account_info();
        let current_match = match_pda.current_match.clone();
        let player_pda_balance = player.try_lamports()?; 

        if signer.key != player.key{
            return err!(ErrorCode::RestrictionError);
        }

        let index = current_match.players.iter().position(|p| p.pub_key == player.key().to_string());
        if index.is_some(){ //// we found a player
            let player_index = index.unwrap();
            let current_match_players = current_match.players.clone();
            let mut find_player = current_match_players[player_index].clone();
            if player_pda_balance > 0{
                **user_pda.try_borrow_mut_lamports()? -= amount;
                **player.try_borrow_mut_lamports()? += amount;
            } else{
                return err!(ErrorCode::PlayerBalanceIsZero);
            }
        } else{
            return err!(ErrorCode::PlayerDoesntExist);
        }

        Ok(())
        
    }

    pub fn deposit(ctx: Context<DepositToMatchPda>, amount: u64) -> Result<()>{

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

        **user_pda_account.try_borrow_mut_lamports()? -= amount;
        **match_pda_account.try_borrow_mut_lamports()? += amount;

        Ok(())

    }

    pub fn start_game(ctx: Context<StartGame>, players: Vec<PlayerInfo>, bump: u8,
                      rounds: i32, size: i32, match_id: String, announce_commit: String) -> Result<()>
    {

        let announced_values = create_announced_values(size, rounds, announce_commit);
        let server = &ctx.accounts.server;
        let server_pda = &mut ctx.accounts.match_pda; // a mutable pointer to the match pda since ctx.accounts fields doesn't implement Copy trait 
        
        let mut players_data = vec![]; 
        for player in players{

            let player_instance = Player{
                pub_key: player.pub_key.to_string(),
                table: create_table(size, player.commit) // creating the table with the passed in size 
            };

            players_data.push(player_instance);
        }

        let current_match = CurrentMatch{
            match_id: match_id.clone(),
            bump,
            server: server.key(),
            is_locked: false,
            announced_values: announced_values.clone(), 
            players: players_data.clone()
        };

        server_pda.current_match = current_match; //// updating the current_match field inside the PDA 


        msg!("{:#?}", CurrentMatchEvent{ 
            match_id: match_id.clone(),  
            server: server.key(), 
            is_locked: false, 
            announced_values: announced_values.clone(), 
            players: players_data.clone() 
        });

        emit!(CurrentMatchEvent{ 
            match_id: match_id.clone(),  
            server: server.key(), 
            is_locked: false, 
            announced_values: announced_values.clone(), 
            players: players_data.clone(),
        });

        Ok(())
        
    }

    pub fn finish_game(ctx: Context<FinishGame>, player_bumps: Vec<u16>) -> Result<()>{ 
        
        //// since Copy traint is not implemented for ctx.accounts fields
        //// like AccountInfo and Account we must borrow the ctx and because 
        //// AccountInfo and Account fields don't imeplement Copy trait 
        //// we must borrow their instance if we want to move them or 
        //// call a method that takes the ownership of their instance 
        //// like unwrap() in order not to be moved. 


        let match_pda = &ctx.accounts.match_pda;
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
        
        let mut winner_count = 0;
        let current_match_pda_amout = **ctx.accounts.match_pda.try_borrow_lamports()?;
        if current_match_pda_amout > 0{

            let winner_flags = winners
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
            
            let winner_reward = current_match_pda_amout / winner_count; //// spread between winners equally

            for is_winner in winner_flags{
                //// every element inside winner_flags is a boolean map to the winner index inside the winners 
                //// vector also player accounts are behind a shared reference thus we can't move out of them
                //// since unwrap(self) method takes the ownership of the type and return the Self because 
                //// in its first param doesn't borrow the self or have &self, the solution is to use a borrow 
                //// of the player account then unwrap() the borrow type like first_player_account.as_ref().unwrap()
                //// with this way we don't lose the ownership of the first_player_account and we can call 
                //// the to_account_info() method on it.
                if is_winner{
                    let winner_account = first_player_account.as_ref().unwrap().to_account_info();
                    **winner_account.try_borrow_mut_lamports()? += winner_reward;
                    **match_pda.try_borrow_mut_lamports()? -= winner_reward;
                }
            }

        } else{
            return err!(ErrorCode::MatchPdaIsEmpty);
        }

        Ok(())

    }

}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Cell{
   pub x: i32,
   pub y: i32,
   pub value: i32,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Player{
   pub pub_key: String,
   pub table: Vec<Cell>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct PlayerInfo{
   pub pub_key: String,
   pub commit: String
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Round{
    pub round_val: i32, 
    pub values: Vec<i32>,
}


#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CurrentMatch{
   pub match_id: String,
   pub bump: u8,
   pub server: Pubkey,
   pub is_locked: bool,
   pub announced_values: Vec<Round>,
   pub players: Vec<Player>,
}


#[account]
pub struct MatchPda{
    current_match: CurrentMatch,
}


#[derive(Accounts)]
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
   pub user_pda: AccountInfo<'info>,
   #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
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
   pub user_pda: AccountInfo<'info>,
   #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
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
   #[account(init, payer = signer, space = 300, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct StartGame<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// only server
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   #[account(init, payer = signer, space = 300, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(player_bump: u8)]
pub struct WithdrawFromUserPda<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only player
   /// CHECK:
    #[account(mut, seeds = [b"ognils", player.key().as_ref()], bump = player_bump)]
    pub user_pda: AccountInfo<'info>,
    #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), 
                            match_pda.current_match.server.key().as_ref()], 
                            bump = match_pda.current_match.bump)]
    pub match_pda: Account<'info, MatchPda>,
    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(player_bumps: Vec<u8>)]
pub struct FinishGame<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only server
   /// CHECK:
    #[account(init, space = 300, payer = signer, seeds = [b"ognils", server.key().as_ref()], bump)]
    pub match_pda: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub server: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", first_user_pda.key().as_ref()], bump = player_bumps[0] as u8)]
    pub first_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", second_user_pda.key().as_ref()], bump = player_bumps[1] as u8)]
    pub second_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", third_user_pda.key().as_ref()], bump = player_bumps[2] as u8)]
    pub third_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", fourth_user_pda.key().as_ref()], bump = player_bumps[3] as u8)]
    pub fourth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", fifth_user_pda.key().as_ref()], bump = player_bumps[4] as u8)]
    pub fifth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"ognils", sixth_user_pda.key().as_ref()], bump = player_bumps[5] as u8)]
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
}


#[event]
#[derive(Debug)]
pub struct CurrentMatchEvent{
    pub match_id: String,
    pub server: Pubkey,
    pub is_locked: bool,
    pub announced_values: Vec<Round>,
    pub players: Vec<Player>,
}
