use crate::error::NftVoterError;
use crate::state::*;
use anchor_lang::prelude::*;


#[derive(Accounts)]
#[instruction(proposal: Pubkey, collection: Pubkey, collection_size: u32)]
pub struct CreateVoteStateForProposal<'info> {
    
    #[account(
        init_if_needed,
        seeds = [ b"voted-nfts".as_ref(),
                proposal.key().as_ref(),
                collection.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + 32 + 32 + 4 + 1 * ((collection_size as usize + 7) / 8)
    )]
    pub voted_nfts: Account<'info, VotedNfts>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}


pub fn create_vote_state_for_proposal(ctx: Context<CreateVoteStateForProposal>, proposal: Pubkey, collection: Pubkey, collection_size: u32) -> Result<()> {
    let voted_nfts = &mut ctx.accounts.voted_nfts;

    require!(
        voted_nfts.voted.len() == 0,
        NftVoterError::VoteStateAlreadyInitialized
    );
    
    voted_nfts.voted = vec![0; (collection_size as usize + 7) / 8];
    voted_nfts.collection = collection;
    voted_nfts.proposal = proposal;
    
    Ok(())
}
