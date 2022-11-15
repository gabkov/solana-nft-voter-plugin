use crate::program_test::nft_voter_test::ConfigureCollectionArgs;
use gpl_nft_voter::error::NftVoterError;
use gpl_nft_voter::state::index_from_nft_literal_name;
use program_test::nft_voter_test::{CastNftVoteArgs, NftVoterTest};
use program_test::tools::{assert_gov_err, assert_nft_voter_err};
use solana_program_test::*;
use solana_sdk::transport::TransportError;
use spl_governance::error::GovernanceError;

use bitvec::prelude::*;

mod program_test;

#[tokio::test]
async fn test_relinquish_nft_vote() -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1, size: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_name = "My Cool NFT #0001 ";

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2_with_specified_name(&nft_collection_cookie, &voter_cookie, None, nft_name)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    nft_voter_test.bench.advance_clock().await;

    // Act

    nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await?;

    // Assert

    let voter_weight_record = nft_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // check if the voted nft position in the bitmap is set back to 0
    let mut voted_nfts = nft_voter_test
        .get_voted_nfts(&voted_nfts_cookie.address)
        .await;

    let nft_idx = index_from_nft_literal_name(nft_name.to_string()).unwrap(); // grabbing the index from the string literal since there is no way to get it from the metadata since it is not an AccountInfo just a PubKey

    let voted = voted_nfts.voted.view_bits_mut::<Lsb0>();

    let vote = voted[nft_idx.checked_sub(1).unwrap() as usize];
    assert_eq!(vote, false, "Vote tracking not set back to false");

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_for_proposal_in_voting_state() -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: 10,
                size: 20,
            }),
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_name = "My Cool NFT #0011 ";

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2_with_specified_name(&nft_collection_cookie, &voter_cookie, None, nft_name)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    // Relinquish Vote from spl-gov
    nft_voter_test
        .governance
        .relinquish_vote(
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
        )
        .await?;

    nft_voter_test.bench.advance_clock().await;

    // Act

    nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await?;

    // Assert

    let voter_weight_record = nft_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // check if the voted nft position in the bitmap is set back to 0
    let mut voted_nfts = nft_voter_test
        .get_voted_nfts(&voted_nfts_cookie.address)
        .await;

    let nft_idx = index_from_nft_literal_name(nft_name.to_string()).unwrap(); // grabbing the index from the string literal since there is no way to get it from the metadata since it is not an AccountInfo just a PubKey

    let voted = voted_nfts.voted.view_bits_mut::<Lsb0>();

    let vote = voted[nft_idx.checked_sub(1).unwrap() as usize];
    assert_eq!(vote, false, "Vote tracking not set back to false");

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_for_proposal_in_voting_state_and_vote_record_exists_error(
) -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: 10,
                size: 20,
            }),
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2(&nft_collection_cookie, &voter_cookie, None)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    // Act

    let err = nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_nft_voter_err(err, NftVoterError::VoteRecordMustBeWithdrawn);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_invalid_voter_error() -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1, size: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_name = "My Cool NFT #0001 ";

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2_with_specified_name(&nft_collection_cookie, &voter_cookie, None, nft_name)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    // Try to use a different voter
    let voter_cookie2 = nft_voter_test.bench.with_wallet().await;

    // Act

    let err = nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie2,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert

    assert_gov_err(err, GovernanceError::GoverningTokenOwnerOrDelegateMustSign);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_unexpired_vote_weight_record() -> Result<(), TransportError>
{
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: 10,
                size: 20,
            }),
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2(&nft_collection_cookie, &voter_cookie, None)
        .await?;

    let args = CastNftVoteArgs {
        cast_spl_gov_vote: false,
    };

    // Cast vote with NFT
    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            Some(args),
        )
        .await?;

    // Act

    let err = nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert

    assert_nft_voter_err(err, NftVoterError::VoterWeightRecordMustBeExpired);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_invalid_voter_weight_token_owner_error(
) -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: 10,
                size: 20,
            }),
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2(&nft_collection_cookie, &voter_cookie, None)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    // Try to update VoterWeightRecord for different governing_token_owner
    let voter_cookie2 = nft_voter_test.bench.with_wallet().await;
    let voter_weight_record_cookie2 = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie2)
        .await?;

    // Act

    let err = nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie2,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await
        .err()
        .unwrap();

    // Assert

    assert_nft_voter_err(err, NftVoterError::InvalidTokenOwnerForVoterWeightRecord);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_using_delegate() -> Result<(), TransportError> {
    // Arrange
    let mut nft_voter_test = NftVoterTest::start_new().await;

    let realm_cookie = nft_voter_test.governance.with_realm().await?;

    let registrar_cookie = nft_voter_test.with_registrar(&realm_cookie).await?;

    let nft_collection_cookie = nft_voter_test.token_metadata.with_nft_collection().await?;

    let max_voter_weight_record_cookie = nft_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_config_cookie = nft_voter_test
        .with_collection(
            &registrar_cookie,
            &nft_collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1, size: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_cookie = nft_voter_test.bench.with_wallet().await;

    let voter_token_owner_record_cookie = nft_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = nft_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = nft_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;
        
    let voted_nfts_cookie = nft_voter_test
        .with_vote_state_for_proposal_and_collection(&proposal_cookie, &collection_config_cookie)
        .await?;

    let nft_name = "My Cool NFT #0001 ";

    let nft_cookie1 = nft_voter_test
        .token_metadata
        .with_nft_v2_with_specified_name(&nft_collection_cookie, &voter_cookie, None, nft_name)
        .await?;

    nft_voter_test
        .cast_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
            None,
        )
        .await?;

    nft_voter_test.bench.advance_clock().await;

    // Setup delegate
    let delegate_cookie = nft_voter_test.bench.with_wallet().await;
    nft_voter_test
        .governance
        .set_governance_delegate(
            &realm_cookie,
            &voter_token_owner_record_cookie,
            &voter_cookie,
            &Some(delegate_cookie.address),
        )
        .await;

    // Act

    nft_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &delegate_cookie,
            &voter_token_owner_record_cookie,
            &[&nft_cookie1],
            &nft_collection_cookie,
            &voted_nfts_cookie,
        )
        .await?;

    // Assert

    let voter_weight_record = nft_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // check if the voted nft position in the bitmap is set back to 0
    let mut voted_nfts = nft_voter_test
        .get_voted_nfts(&voted_nfts_cookie.address)
        .await;

    let nft_idx = index_from_nft_literal_name(nft_name.to_string()).unwrap(); // grabbing the index from the string literal since there is no way to get it from the metadata since it is not an AccountInfo just a PubKey

    let voted = voted_nfts.voted.view_bits_mut::<Lsb0>();

    let vote = voted[nft_idx.checked_sub(1).unwrap() as usize];
    assert_eq!(vote, false, "Vote tracking not set back to false");

    Ok(())
}
