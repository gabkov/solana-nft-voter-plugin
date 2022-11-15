use crate::error::NftVoterError;
use crate::{state::*};
use anchor_lang::prelude::*;
use anchor_lang::Accounts;
use itertools::Itertools;
use bitvec::prelude::*;

/// Casts NFT vote. The NFTs used for voting are tracked using NftVoteRecord accounts
/// This instruction updates VoterWeightRecord which is valid for the current Slot and the target Proposal only
/// and hance the instruction has to be executed inside the same transaction as spl-gov.CastVote
///
/// CastNftVote is accumulative and can be invoked using several transactions if voter owns more than 5 NFTs to calculate total voter_weight
/// In this scenario only the last CastNftVote should be bundled  with spl-gov.CastVote in the same transaction
///
/// CastNftVote instruction and NftVoteRecord are not directional. They don't record vote choice (ex Yes/No)
/// VoteChoice is recorded by spl-gov in VoteRecord and this CastNftVote only tracks voting NFTs
///
#[derive(Accounts)]
#[instruction(proposal: Pubkey, collection: Pubkey)]
pub struct CastNftVote<'info> {
    /// The NFT voting registrar
    pub registrar: Account<'info, Registrar>,

    #[account(
        mut,
        constraint = voter_weight_record.realm == registrar.realm
        @ NftVoterError::InvalidVoterWeightRecordRealm,

        constraint = voter_weight_record.governing_token_mint == registrar.governing_token_mint
        @ NftVoterError::InvalidVoterWeightRecordMint,
    )]
    pub voter_weight_record: Account<'info, VoterWeightRecord>,

    /// TokenOwnerRecord of the voter who casts the vote
    #[account(
        owner = registrar.governance_program_id
     )]
    /// CHECK: Owned by spl-governance instance specified in registrar.governance_program_id
    voter_token_owner_record: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = voted_nfts.proposal == proposal @ NftVoterError::InvalidVotedNftsProposal,
        constraint = voted_nfts.collection == collection @ NftVoterError::InvalidVotedNftsCollection,
    )]
    pub voted_nfts: Account<'info, VotedNfts>,

    /// Authority of the voter who casts the vote
    /// It can be either governing_token_owner or its delegate and must sign this instruction
    pub voter_authority: Signer<'info>,

    /// The account which pays for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Casts vote with the NFT
pub fn cast_nft_vote<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CastNftVote<'info>>,
    proposal: Pubkey,
    _collection: Pubkey
) -> Result<()> {
    let registrar = &ctx.accounts.registrar;
    let voter_weight_record = &mut ctx.accounts.voter_weight_record;

    let governing_token_owner = resolve_governing_token_owner(
        registrar,
        &ctx.accounts.voter_token_owner_record,
        &ctx.accounts.voter_authority,
        voter_weight_record,
    )?;

    let voted_nfts = &mut ctx.accounts.voted_nfts;
    

    let mut voter_weight = 0u64;

    // Ensure all voting nfts in the batch are unique
    let mut unique_nft_mints = vec![];


    for (nft_info, nft_metadata_info) in
        ctx.remaining_accounts.iter().tuples()
    {
        let (nft_vote_weight, _, nft_index) = resolve_nft_vote_weight_and_mint_and_nft_index(
            registrar,
            &governing_token_owner,
            nft_info,
            nft_metadata_info,
            &mut unique_nft_mints,
        )?;

        voter_weight = voter_weight.checked_add(nft_vote_weight as u64).unwrap();

            
        let voted = voted_nfts.voted.view_bits_mut::<Lsb0>();

        let voter_index = nft_index.checked_sub(1)
            .ok_or(NftVoterError::ArithMeticError)? as usize;

        if voted[voter_index] {
            return Err(NftVoterError::NftAlreadyVoted.into());
        }

        voted.set(voter_index, true);
        
    }

    if voter_weight_record.weight_action_target == Some(proposal)
        && voter_weight_record.weight_action == Some(VoterWeightAction::CastVote)
    {
        // If cast_nft_vote is called for the same proposal then we keep accumulating the weight
        // this way cast_nft_vote can be called multiple times in different transactions to allow voting with any number of NFTs
        voter_weight_record.voter_weight = voter_weight_record
            .voter_weight
            .checked_add(voter_weight)
            .unwrap();
    } else {
        voter_weight_record.voter_weight = voter_weight;
    }

    // The record is only valid as of the current slot
    voter_weight_record.voter_weight_expiry = Some(Clock::get()?.slot);

    // The record is only valid for casting vote on the given Proposal
    voter_weight_record.weight_action = Some(VoterWeightAction::CastVote);
    voter_weight_record.weight_action_target = Some(proposal);

    Ok(())
}
