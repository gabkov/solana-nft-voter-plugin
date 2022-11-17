# nation nft-voter-plugin

Based on [governance-program-library](https://github.com/solana-labs/governance-program-library) nft-voter plugin.

## To run the tests
```console
RUST_LOG=none cargo test-bpf -- --nocapture
```

## The main difference between the original nft-plugin and this version

The only thing changed is how the already voted NFTs gets counted. Originally for each NFT vote a new NFTVoteRecord was created which required to create a new account. Once the voting was done, this account can be disposed and recollect the rent from these NFTVoteRecords, with a second interaction.

The new implementation, holds the NFT votes in a seprarate account for each proposal/collection in a bit slice, where each NFT vote is tracked by the NFT id. So let's say the NFT with the name "My NFT #011", would mean that in our bit slice the 10th element is set to true. Same logic applies for relinquish vote, but in a reverse order. 

With this implementation the users, won't have to initiate a new transaction to recollect the rent from the NftVoteRecords, so the voting is only one interaction. 