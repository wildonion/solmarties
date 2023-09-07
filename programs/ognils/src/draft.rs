

// SPL token version





use anchor_spl::{token::TokenAccount, token::Transfer};
use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("YRDxsg529tECpHUToZ61uMfUeWF2fDCzLeJoLNq4dFt");



#[program]
pub mod ognils {


    use anchor_spl::token::{self, transfer};

    use super::*;
    
    pub fn init_match_pda(ctx: Context<InitMatchPda>, match_id: String, bump: u8) -> Result<()>{

        let server = &ctx.accounts.spltoken_server;
        let match_pda_data = &mut ctx.accounts.match_pda;

        if &ctx.accounts.signer.key() != &server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        match_pda_data.match_id = match_id.clone();
        match_pda_data.bump = bump;
        match_pda_data.server = server.owner.key();

        Ok(())

    }

    pub fn deposit(ctx: Context<DepositToMatchPda>, match_id: String, amount: u64) -> Result<()>{

        if ctx.accounts.spltoken_user.amount < amount {
            return err!(ErrorCode::InsufficientFund);
        }


        /* transfer amount from spltoken pda to spltoken match pda */
        transfer(
            CpiContext::new(
                ctx.accounts.spltoken_token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.spltoken_user.to_account_info(),
                    to: ctx.accounts.spltoken_match_server.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            amount,
        )?;

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

        let server = &ctx.accounts.spltoken_server;
        let server_pda = &mut ctx.accounts.match_pda;
        let signer = &ctx.accounts.signer;

        if signer.key != &server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        if server.amount < bet_value{
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
            server_payback: u64
        ) -> Result<()>{ 
        
        //// since Copy traint is not implemented for ctx.accounts fields
        //// like AccountInfo and Account we must borrow the ctx and because 
        //// AccountInfo and Account fields don't imeplement Copy trait 
        //// we must borrow their instance if we want to move them or 
        //// call a method that takes the ownership of their instance 
        //// like unwrap() in order not to be moved. 

        let match_pda_data = &ctx.accounts.match_pda;
        let signer = ctx.accounts.signer.key;
        let server = ctx.accounts.spltoken_match_server.owner.key();

        let server_account = ctx.accounts.spltoken_match_server.to_account_info();


        if signer != &ctx.accounts.spltoken_match_server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        // ----------------- players accounts ----------------------
        //// can't move out of a type if it's behind a shread reference
        //// if there was Some means we have winners
        let first_player_account = &ctx.accounts.spltoken_first_user;
        let second_player_account = &ctx.accounts.spltoken_second_user;
        let third_player_account = &ctx.accounts.spltoken_third_user;
        let fourth_player_account = &ctx.accounts.spltoken_fourth_user;
        let fifth_player_account = &ctx.accounts.spltoken_fifth_user;
        let sixth_player_account = &ctx.accounts.spltoken_sixth_user;
        let winners = vec![first_player_account, second_player_account, third_player_account,
                                                     fourth_player_account, fifth_player_account, sixth_player_account];
        
        {
            let mut winner_count = 0;
            let current_match_pda_amout = ctx.accounts.spltoken_match_server.amount;
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
                let payback_server_amount = server_payback / winner_count; //// spread between winners equally
                let remaining_in_pda = current_match_pda_amout - winner_reward;
                

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

                        transfer(
                            CpiContext::new(
                                ctx.accounts.spltoken_token_program.to_account_info(),
                                Transfer {
                                    from: ctx.accounts.spltoken_match_server.to_account_info(),
                                    to: winner_account.clone(),
                                    authority: ctx.accounts.signer.to_account_info(),
                                },
                            ),
                            winner_reward,
                        )?;


                        transfer(
                            CpiContext::new(
                                ctx.accounts.spltoken_token_program.to_account_info(),
                                Transfer {
                                    from: winner_account.clone(),
                                    to: ctx.accounts.spltoken_match_server.to_account_info(),
                                    authority: winner_account,
                                },
                            ),
                            payback_server_amount,
                        )?;

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
            server: ctx.accounts.spltoken_match_server.owner.key(), 
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


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct DepositToMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
  
   /// CHECK:
   #[account(mut)]
   pub spltoken_match_server: Account<'info, anchor_spl::token::TokenAccount>,
   /// CHECK:
   #[account(mut)]
   pub spltoken_user: Account<'info, anchor_spl::token::TokenAccount>,

   #[account(init, payer = signer, space = 1024, seeds = [match_id.as_bytes(), spltoken_match_server.owner.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,

   pub system_program: Program<'info, System>,
   pub spltoken_token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct InitMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   /// CHECK:
   #[account(mut)]
   pub spltoken_server: Account<'info, TokenAccount>,
   #[account(init, payer = signer, space = 1024, seeds = [match_id.as_bytes(), spltoken_server.owner.key().as_ref()], bump)]
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
   pub spltoken_server: Account<'info, TokenAccount>,
   #[account(mut, seeds = [match_id.as_bytes(), 
                            match_pda.server.key().as_ref()], 
                            bump = bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct FinishGame<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only server
   /// CHECK:
   #[account(mut, seeds = [match_id.as_bytes(), 
                        spltoken_match_server.owner.key().as_ref()], 
                            bump = match_pda.bump)]
    pub match_pda: Box<Account<'info, MatchPda>>,
    // ------------------------------ spltoken PDAs ---------------------
    /// CHECK:
    #[account(mut)]
    pub spltoken_match_server: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_first_user: Option<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_second_user: Option<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_third_user: Option<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_fourth_user: Option<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_fifth_user: Option<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub spltoken_sixth_user: Option<Account<'info, TokenAccount>>,
    // ---------------------------------------------------------------
    pub system_program: Program<'info, System>,
    pub spltoken_token_program: AccountInfo<'info>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Restriction error!")]
    RestrictionError,
    #[msg("Player Doesn't Exist!")]
    PlayerDoesntExist,
    #[msg("Player Pda Balance Is Zero!")]
    PlayerPdaBalanceIsZero,
    #[msg("Server Pda Balance Is Zero!")]
    ServerPdaBalanceIsZero,
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


