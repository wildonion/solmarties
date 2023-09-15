

// SPL token version










use anchor_spl::{token::TokenAccount, token::{self, Mint}, token::{Transfer, Token}};
use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("YRDxsg529tECpHUToZ61uMfUeWF2fDCzLeJoLNq4dFt");



#[program]
pub mod slingo {


    use anchor_spl::token::{self, transfer};

    use super::*;
    
    pub fn init_match_pda(ctx: Context<InitMatchPda>, match_id: String, bump: u8) -> Result<()>{

        let server = &ctx.accounts.anyspl_server;
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

        if ctx.accounts.anyspl_match_server.amount < amount {
            return err!(ErrorCode::InsufficientFund);
        }

        // users must transfer anyspl to matchtokenpda
        // matchtokenpda = server pubkey + anyspl server
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.anyspl_user.to_account_info(),
                    to: ctx.accounts.matchtokenpda.to_account_info(),
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

        let server = &ctx.accounts.anyspl_server;
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
        if signer != &ctx.accounts.anyspl_match_server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        // ----------------- players accounts ----------------------
        //// can't move out of a type if it's behind a shread reference
        //// if there was Some means we have winners
        let first_player_account = &ctx.accounts.anyspl_first_user;
        let second_player_account = &ctx.accounts.anyspl_second_user;
        let third_player_account = &ctx.accounts.anyspl_third_user;
        let fourth_player_account = &ctx.accounts.anyspl_fourth_user;
        let fifth_player_account = &ctx.accounts.anyspl_fifth_user;
        let sixth_player_account = &ctx.accounts.anyspl_sixth_user;
        let winners = vec![first_player_account, second_player_account, third_player_account,
                                                     fourth_player_account, fifth_player_account, sixth_player_account];
        
        {
            let mut winner_count = 0;
            let current_match_pda_amout = ctx.accounts.anyspl_match_server.amount;
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
                                ctx.accounts.token_program.to_account_info(),
                                Transfer {
                                    from: ctx.accounts.matchtokenpda.to_account_info(),
                                    to: winner_account.clone(),
                                    authority: ctx.accounts.signer.to_account_info(),
                                },
                            ),
                            winner_reward,
                        )?;


                        transfer(
                            CpiContext::new(
                                ctx.accounts.token_program.to_account_info(),
                                Transfer {
                                    from: winner_account.clone(),
                                    to: ctx.accounts.anyspl_match_server.to_account_info(),
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
            server: ctx.accounts.anyspl_match_server.owner.key(), 
            server_commit: match_pda_data.server_commit.clone(), 
            bet_value: match_pda_data.bet_value, 
            bump: match_pda_data.bump, 
            server_key: match_pda_data.server_key.clone(), 
            ipfs_link: match_pda_data.ipfs_link.clone(), 
            chain_table: match_pda_data.chain_table 
        });

        Ok(())

    }


    pub fn initialize_match_token_pda(ctx: Context<InitMatchTokenPda>, match_id: String, _bump1:u8) -> Result<()> {

        msg!("token got Initialised");
        let pda = ctx.accounts.matchtokenpda.key();
        msg!("token pda : {}", pda);
        Ok(())
    
    }

    pub fn withdraw_from_match_token_pda(ctx: Context<WithdrawMatchTokenPda>, match_id: String, _bump1: u8, amount: u64) -> Result<()>{

        if ctx.accounts.matchtokenpda.amount < amount {
            return err!(ErrorCode::InsufficientFund);
        }

        let server = ctx.accounts.anyspl_match_server.to_account_info();
        let signer = ctx.accounts.signer.to_account_info();

        if signer.key != &server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        // users must transfer anyspl to matchtokenpda
        // matchtokenpda = server pubkey + anyspl server
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.matchtokenpda.to_account_info(),
                    to: ctx.accounts.anyspl_match_server.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
        
    }

}


#[derive(Accounts)]
#[instruction(match_id: String, _bump1: u8)]
pub struct WithdrawMatchTokenPda<'info> {

    #[account(mut)]
   pub signer: Signer<'info>, //// server 
  
   pub matchtokenpda: Account<'info, TokenAccount>,

    /// CHECK:
   pub mint: Account<'info, Mint>,
   /// CHECK:
   #[account(mut)]
   pub anyspl_match_server: Account<'info, anchor_spl::token::TokenAccount>,

   /// CHECK:
   #[account(mut)]
   pub anyspl_user: Account<'info, anchor_spl::token::TokenAccount>,

   #[account(init, payer = signer, space = 1024, seeds = [match_id.as_bytes(), anyspl_match_server.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,

   pub system_program: Program<'info, System>,
   /// CHECK: 
   pub token_program: Program<'info, Token>,


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
#[instruction(match_id: String, _bump : u8)]
pub struct InitMatchTokenPda<'info> { // server must call this

    // init matchtokenpda : server public key + anyspl server
    #[account(
        init,
        seeds = [match_id.as_bytes(), owner.key.as_ref(), anyspl_match_server.key().as_ref()],
        bump,
        payer = owner,
        token::mint = mint,
        token::authority = match_pda,
     )]
     
    pub matchtokenpda: Account<'info, TokenAccount>,
    /// CHECK:
    pub mint: Account<'info, Mint>,
    /// CHECK:
    pub match_pda: Box<Account<'info, MatchPda>>,
    /// CHECK:
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_match_server: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    /// CHECK:
    pub token_program: Program<'info, Token>,
   
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct DepositToMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server 
  
   pub matchtokenpda: Account<'info, TokenAccount>,

    /// CHECK:
   pub mint: Account<'info, Mint>,
   /// CHECK:
   #[account(mut)]
   pub anyspl_match_server: Account<'info, anchor_spl::token::TokenAccount>,

   /// CHECK:
   #[account(mut)]
   pub anyspl_user: Account<'info, anchor_spl::token::TokenAccount>,

   #[account(init, payer = signer, space = 1024, seeds = [match_id.as_bytes(), anyspl_match_server.key().as_ref()], bump)]
   pub match_pda: Box<Account<'info, MatchPda>>,

   pub system_program: Program<'info, System>,
   /// CHECK: 
   pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct InitMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   /// CHECK:
   #[account(mut)]
   pub anyspl_server: Account<'info, TokenAccount>,
   #[account(init, payer = signer, space = 1024, seeds = [match_id.as_bytes(), anyspl_server.key().as_ref()], bump)]
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
   pub anyspl_server: Account<'info, TokenAccount>,
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
                        anyspl_match_server.key().as_ref()], 
                            bump = match_pda.bump)]
    pub match_pda: Box<Account<'info, MatchPda>>,
    pub matchtokenpda: Box<Account<'info, TokenAccount>>,
    // ------------------------------ JELLY PDAs ---------------------
    /// CHECK:
    #[account(mut)]
    pub anyspl_match_server: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_first_user: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_second_user: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_third_user: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_fourth_user: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_fifth_user: Option<Box<Account<'info, TokenAccount>>>,
    /// CHECK:
    #[account(mut)]
    pub anyspl_sixth_user: Option<Box<Account<'info, TokenAccount>>>,
    // ---------------------------------------------------------------
    pub system_program: Program<'info, System>,
   /// CHECK:
    pub token_program: AccountInfo<'info>,
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


