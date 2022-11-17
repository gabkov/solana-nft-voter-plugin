use anchor_lang::prelude::*;
use mpl_token_metadata::state::Metadata;

use crate::{id, error::NftVoterError};

/// Holds the already voted NFTs in a vec. The vec is read as a mutable bit slice so once an nft is voted the corresponding 
/// bit at the index of the NFT id will be set to true.
/// Example: [0, 0, 0, 1, 0] -> this means the NFT with id #05 voted
#[account]
#[derive(Debug, PartialEq, Default)]
pub struct VotedNfts {

    pub proposal: Pubkey,

    pub collection: Pubkey,

    pub voted: Vec<u8>,
}

/// Returns VotedNfts PDA seeds
pub fn get_voted_nfts_seed<'a>(proposal: &'a Pubkey, collection: &'a Pubkey) -> [&'a [u8]; 3] {
    [b"voted-nfts", proposal.as_ref(), collection.as_ref()]
}

/// Returns VotedNfts PDA address
pub fn get_voted_nfts_address(proposal: &Pubkey, collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&get_voted_nfts_seed(proposal, collection), &id()).0
}


pub fn index_from_nft_name(metadata: &Metadata) -> Result<u64> {

    let name = &metadata.data.name;
    let num_idx = name.find(
        |c: char| c.is_ascii_digit()).ok_or(NftVoterError::InvalidMetadataName)?;

    let name = &name[num_idx..];

    // TODO: could overflow but lol
    let end_idx = name.find(
        |c: char| !c.is_ascii_digit()).ok_or(NftVoterError::InvalidMetadataName)?;

    let idx = name[..end_idx].parse::<u64>()
        .map_err(|_| NftVoterError::InvalidMetadataName)?;

    msg!("Parsed index {} from name {}", idx, &metadata.data.name);

    Ok(idx)
}

/// Used as a helper for tests
pub fn index_from_nft_literal_name(full_nft_name: String) -> Result<u64> {

    let name = &full_nft_name;
    let num_idx = name.find(
        |c: char| c.is_ascii_digit()).ok_or(NftVoterError::InvalidMetadataName)?;

    let name = &name[num_idx..];

    // TODO: could overflow but lol
    let end_idx = name.find(
        |c: char| !c.is_ascii_digit()).ok_or(NftVoterError::InvalidMetadataName)?;

    let idx = name[..end_idx].parse::<u64>()
        .map_err(|_| NftVoterError::InvalidMetadataName)?;

    println!("Parsed index {} from name {}", idx, full_nft_name);

    Ok(idx)
}

