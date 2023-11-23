# Stargaze Token Bound Accounts

Stargaze Production Contracts implementing [cw82](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw82) and [cw83](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw83) 


## Token Registry

Token-Bound Account Registry that follows [cw83](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw83) account registry standard.  

Allows users to create an abstract account linked to an NFT, to reset it or to migrate it to a newer code version if the user is a current owner of that token

Has a future-reserved `freeze` `unfreeze` primitives that can help against fraudulent asset manipulation. Read a section from [ERC-6551](https://eips.ethereum.org/EIPS/eip-6551#fraud-prevention). Admin list must be initialzied to a neutral entity or an infrastructure contract to prevent centralization

Provides additional query endpoints to get all the existing accounts, all kwown collections and accounts linked to a token of a specific collection. 

## Token Account

A smart contract based account following [cw82](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw82). 

Uses a secp256k1 public key and support arbitrary signature verification defined in [036](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-036-arbitrary-signature.md) to implement [cw81](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw81)

As a minimal proxy following [cw1](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw1) allows to initiate messages with the smart account as an initiator (msg.sender) but only to the owner of a token the account is linked to. Ownership verification happens through the registry and the new owner must claim the account through it to get access.

The contract implementation allows the accoubt to become aware of other tokens in its posession. That allows users to directly query the asset inventory  and gives a flexibilty for the owner to explicitely associate itself with certain tokens

### Note
To be released under fully open-source license once released. Strict all-right reserved until them while in development


