


// spl token


use anchor_spl::{token::TokenAccount, token::{self, Mint}, token::{Transfer, Token}};
use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("YRDxsg529tECpHUToZ61uMfUeWF2fDCzLeJoLNq4dFt");


#[program]
pub mod ognils {


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

        let match_pda_data = &ctx.accounts.match_pda;
        let signer = ctx.accounts.signer.key;
        if signer != &ctx.accounts.anyspl_match_server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        // ----------------- players accounts ----------------------
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
            let current_matchtokenpda_amout = ctx.accounts.matchtokenpda.amount;
            if current_matchtokenpda_amout > 0 && current_matchtokenpda_amout >= match_pda_data.bet_value{

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
                
                let winner_reward = (match_pda_data.bet_value - server_payback) / winner_count;
                let bump_vector = match_pda_data.bump.to_le_bytes();
                let dep = &mut ctx.accounts.anyspl_match_server.key();
                let inner = vec![match_id.as_ref(), dep.as_ref(), bump_vector.as_ref()];
                let outer = vec![inner.as_slice()];
                
                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.matchtokenpda.to_account_info(),
                            to: ctx.accounts.anyspl_match_server.to_account_info().clone(),
                            authority: ctx.accounts.match_pda.to_account_info(),
                        },
                        outer.as_slice()
                    ),
                    server_payback,
                )?;
                
                ctx.accounts.matchtokenpda.reload()?;

                for is_winner_idx in 0..winner_flags.len(){

                    if winner_flags[is_winner_idx]{
                        let winner = winners[is_winner_idx].clone();
                        let winner_account = winner.as_ref().unwrap().to_account_info();

                        transfer(
                            CpiContext::new_with_signer(
                                ctx.accounts.token_program.to_account_info(),
                                Transfer {
                                    from: ctx.accounts.matchtokenpda.to_account_info(),
                                    to: winner_account.to_account_info().clone(),
                                    authority: ctx.accounts.match_pda.to_account_info(),
                                },
                                outer.as_slice()
                            ),
                            winner_reward,
                        )?;

                    }
                }

            } else{
                return err!(ErrorCode::MatchPdaIsEmpty);
            }
        }

        let match_pda_data = &mut ctx.accounts.match_pda;
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


    pub fn initialize_match_token_pda(ctx: Context<InitMatchTokenPda>, match_id: String) -> Result<()> {

        msg!("matchtokenpda got Initialised");
        let pda = ctx.accounts.matchtokenpda.key();
        msg!("matchtokenpda key : {}", pda);
        Ok(())
    
    }

    pub fn withdraw_from_match_token_pda(ctx: Context<WithdrawMatchTokenPda>, match_id: String, amount: u64) -> Result<()>{

        if ctx.accounts.matchtokenpda.amount < amount {
            return err!(ErrorCode::InsufficientFund);
        }

        let signer = ctx.accounts.signer.key;
        if signer != &ctx.accounts.anyspl_match_server.owner.key(){
            return err!(ErrorCode::RestrictionError);
        }

        let bump_vector = ctx.accounts.match_pda.bump.to_le_bytes();
        let dep = &mut ctx.accounts.anyspl_match_server.key();
        let inner = vec![match_id.as_ref(), dep.as_ref(), bump_vector.as_ref()];
        let outer = vec![inner.as_slice()];
        
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.matchtokenpda.to_account_info(),
                    to: ctx.accounts.anyspl_match_server.to_account_info().clone(),
                    authority: ctx.accounts.match_pda.to_account_info(),
                },
                outer.as_slice()
            ),
            amount,
        )?;

        Ok(())
        
    }

}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct WithdrawMatchTokenPda<'info> {

    #[account(mut)]
   pub signer: Signer<'info>, //// server 

    /// CHECK:
    #[account(mut)]
   pub matchtokenpda: Account<'info, TokenAccount>,

    /// CHECK:
   pub mint: Account<'info, Mint>,
   /// CHECK:
   #[account(mut)]
   pub anyspl_match_server: Account<'info, anchor_spl::token::TokenAccount>,

   /// CHECK:
   #[account(mut)]
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
#[instruction(match_id: String)]
pub struct InitMatchTokenPda<'info> { //// server

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
   pub signer: Signer<'info>, //// server
   /// CHECK:
   #[account(mut)]
   pub anyspl_server: Account<'info, TokenAccount>,
   #[account(mut, seeds = [match_id.as_bytes(), 
                            anyspl_server.key().as_ref()], 
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
    /// CHECK:
    #[account(mut)]
    pub matchtokenpda: Box<Account<'info, TokenAccount>>,
    // ------------------------------ SPLTOKEN ACCOUNTS ---------------------
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
   pub token_program: Program<'info, Token>,
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
