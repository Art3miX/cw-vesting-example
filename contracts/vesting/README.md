# Minter sub
This is a minter contract that goes on sub chain of a collection for an interchain mint.

# Flow
1. User mints NFT on sub-chain and provide it with a TransferStrategy.
2. The Minter verify the mint data, and pass a message to arkite to execute mint on home-chain
3. Arkite does an IBC call to mint on home-chain
4. Minter-main on home-chain mint an NFT
5. Based on provided TransferStrategy, the NFT is transferred to an address on home-chain or IBC back to sub-chain

# Possible errors
The minter-sub have 1 possible error that it handles, the IBC mint call fails.
* Note the transfer is a separate flow unrelated to the minter-sub.

If the mint process fails, Arkite send a message to minter-sub and tell it 2 possible things:

1. Refund - The mint failed, so we should refund the user on sub-chain.
2. Close mint - In case the return error is that the mint on home-chain is finished, we want to also close it on the sub-chain.

* Note - We only close mint if we get an error that the mint on home-chain is finished, otherwise we don't call mint.

# Admin calls

```rust
pub enum AdminMsg {
    /// Update the mint price
    UpdateMintPrice { price: Coin },
    /// Update the start time
    UpdateStartTime(Timestamp),
    /// Update the per address limit
    UpdatePerAddressLimit { per_address_limit: u32 },
    /// Close the mint to stop minting
    CloseMint,
}
```
