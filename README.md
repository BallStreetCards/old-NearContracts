# TokenizedCard Token (NFT)

1. TokenizedCard - Just a normal NFT contract with metadata. Should show up in the NEAR wallet (NFT tab).

2. CardStorefront - Deploys a new TokenizerCard with the following config.

Constructor params
* costPerToken (in $USN)
* totalSupply
* metadata - NFT metadata
* owner - account to receive the $USN
  
Methods 
* buy - will transfer the maximum available number of associated TokenizerCard NFTs that is fully covered by the $USN amount the user has sent. Should send the charged $USN to the owner, and refund the remaining $USN 
* Example1: Card costs $USN 10, user calls buy and sends $USN 43, there are 100 card tokens available => user gets 4 card tokens, and is refunded $USN 3. 
* Example 2: Card costs $USN 21, user calls buy and sends $USN 163, there are 3 card tokens available => user gets 3 card tokens, and is refunded $USN 100.
