# Isolated PIV1 V2 Testnet installation and genesis

**CONFIRMED on Solana Testnet.** The isolated Config V2 release installed at slot **467766588**. Fresh application genesis completed under the new Squads Vault 0, and the official JitoSOL contribution established protected principal. The 967,696-byte stripped ELF read back from ProgramData matched the reviewed source artifact.

| Binding | Value |
| --- | --- |
| Testnet genesis | `4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY` |
| Program | `FxJAXiBCyA6HQhA9T5yhihGJsY4WRAW5XPdsMdhhrDR` |
| ProgramData | `Fr7sxef6YNEbN5k35tV3PCtG6r6CtNRY4MPNFsX2atVZ` |
| Squads V4 multisig | `2fAMcdsTDRbtoGfqhAM5zf4crQFaQFjWPJ5f23UqmeE5` |
| ProgramData authority and canonical Vault 0 | `7h92e5jpcFZSTqtpyUJzKMfA5jVkG6scaMk7dUKXSsbJ` |
| Config | `CyDLgfDe6boxE41o9PHZy5GnvFiByp45gDbNtNLKWGxr` |
| Distribution | `2mdhXtuPErSe9FNwD1fewfTQN5Epsroz9C5nMzTKLnFo` |
| Reviewed source commit | `c7914950242ac07927eed38a5e1dc3cf2e5f13a8` |
| Exact deployed ELF SHA-256 | `2ea454e9eac013bab063651c6691bf1cf84fad3e5d3e0b67375f2603e4138109` |

The build used locked source and Anza platform-tools v1.54 with the recovery-enabled feature set and strict principal rounding. This deployment verifies the exact ELF and authority handover. Config V2 records two distinct Testnet beneficiaries and the confirmed 59% / 19.5% / 19.5% / 2% distribution policy. The six distinct guardian keys were under one VPS operator for this isolated experiment. Four distinct real guardian signatures were verified for each proposal.

| Stage | Finalized slot | Original |
| --- | ---: | --- |
| Create isolated Squads multisig | 467757305 | [Original receipt](https://explorer.solana.com/tx/4gUkVw9TiWAFzkkPsqZf2HgnDjAt4TYVzxRPP3WhNjnxozzmdfbgWUtpKq2KHLXJLMPskbcANJE74B4YhU2YUbMh?cluster=testnet) |
| Deploy ELF and bind Vault 0 | 467766588 | [Original receipt](https://explorer.solana.com/tx/effUCUz372X1nbSSW7M1Adk1J5MLv7UcVf2ns9syRCbvoQxVqFkezQKczCge82KejXBgPeEZMSGYQbGaFnw6HeW?cluster=testnet) |
| Create address lookup table | 467766629 | [Original receipt](https://explorer.solana.com/tx/4rVYDhajPBZhTvdnYk1SxCxHPecW9gig4kSYvHBZ5QpwEhKASbdL1eYAg4DdKrFRVcBKR188HZKeJHcXsuskkbxi?cluster=testnet) |
| Extend address lookup table | 467766656 | [Original receipt](https://explorer.solana.com/tx/3sTR1S4YrYxCYgdFWdNTmYKfCZjRALtJNgNhtwfiZ4AT84dr54VSrbJ2Ynt1eDyWuwdZdkqPLDmLsz6apV6bRkoB?cluster=testnet) |
| IN02 Approval 1 | 467767462 | [Original receipt](https://explorer.solana.com/tx/5xkN55PLwdzG9xHTR1CoNxi5t1KNPgcNkwtJa6S3kcupJUxQXN9RfL9gB2E5X4wC4FKWBEoVvXF7RQfkxTWK3eKj?cluster=testnet) |
| IN02 Approval 2 | 467767492 | [Original receipt](https://explorer.solana.com/tx/5zGJCW5NqRiWMAaKfQ9k8qntEEDvcURRZ5Fo2UEd9Vp1i2Px6eaD4rht4Rxm9vmsQYWtThjgxNEmgiBZfaXk5Bdc?cluster=testnet) |
| IN02 Approval 3 | 467767521 | [Original receipt](https://explorer.solana.com/tx/UWUhvTVtYdsvdnUWWXhNC4nyW5ssR1XLadxgZ5d9UJFvQBjr6QhQjCZcS5u6TLLacRu2f9hkTV8MhtsR2BACukx?cluster=testnet) |
| IN02 Approval 4 | 467767551 | [Original receipt](https://explorer.solana.com/tx/27E7p4qyFgn5L22AUDyD1BYqzZnsDigbMWhTeLoZEaCGT6WzDVX33KpNbzscz6B6Jqj8KnyEBxhcQSe5Ut1Ncjr2?cluster=testnet) |
| IN02 Execute | 467767584 | [Original receipt](https://explorer.solana.com/tx/bKyCbu4gM6Jak4S5hoyUkRDuQxQDuAf1BvZMumrzF48A1LEQcC4pS3VkKrtUASPG1uVRCZNkumez5oenNEcmr1t?cluster=testnet) |
| Fund the measured MR02 shortfall | 467767615 | [Original receipt](https://explorer.solana.com/tx/3k98oFnEp31w8nxxzyhnizziSgTBAqpTkdKonr14M1F3pKdtweF88UYFh32rkVqVtCfqCyPpUQTmSkLoQKNAPDAm?cluster=testnet) |
| Migrate the pristine round to V2 | 467767643 | [Original receipt](https://explorer.solana.com/tx/3bg5S4GUL1PiqG3L9CTJKKUvVo7QEApNGU6YkJsSQ2Wq1X3QiSbHZrhdXZ7BM2KUEwBLCbAwWaN97ji9rbJ4t5HH?cluster=testnet) |
| MR03 Approval 1 | 467767801 | [Original receipt](https://explorer.solana.com/tx/5CyX4mFqxzciVpyWz4sShyW6eJa9rM548WXmjFec5EpN5G6jiHrud8YAKtPxMJoSPuaRNJBVNDCUnebiwxTPLVHa?cluster=testnet) |
| MR03 Approval 2 | 467767830 | [Original receipt](https://explorer.solana.com/tx/2ddQhk8uWYMVX7ruenj8h4kgiCctWFtEbNGcq4kjzzN13Vb8dEUBieFaH4mYem3o7tUi5SbsRXB1bBcMr3Wzz8VU?cluster=testnet) |
| MR03 Approval 3 | 467767860 | [Original receipt](https://explorer.solana.com/tx/5TpQPmsRNLcMv7uFgyHD29EnnLCBwKJqpA3qJCw3dn1Xje9Jz2VTNocW5AmWYR6VEUCJNeEw1EwNZa9fzewvc1SF?cluster=testnet) |
| MR03 Approval 4 | 467767889 | [Original receipt](https://explorer.solana.com/tx/2FeThYkrxTRvXiZtg5MmitQATP8sCwer8Bh99RSGu2fSjittBMrjnnof69xLGkaPzBzEEm8q3QebC2ASUvVYtX3o?cluster=testnet) |
| MR03 Execute | 467767921 | [Original receipt](https://explorer.solana.com/tx/2eGPrKsg9cccpSjsYXfXHCgF529vsEE8QuGLLt6Rj213kji2J63npqScgtmRkWbktsMQeR3uYPsFGDqV6RUeHQ8C?cluster=testnet) |
| Buy official JitoSOL externally | 467767953 | [Original receipt](https://explorer.solana.com/tx/3mPyVEiainpwSMijdk7EtuZy8FTg75C2TtGwkewuuq8ZYt1z25MGWMxkqk1xYyjkdafeKzqcWkDEU998ZH5MXJKY?cluster=testnet) |
| Atomically contribute and integrate JitoSOL | 467767992 | [Original receipt](https://explorer.solana.com/tx/4i9XRcjHBoCuJAyqSUe2E6KFadNEfgmcnQhSsFKG83caT6Eib3Sc6zzaeBm8REdZskCe1Why9ajkYVtgo753wt6r?cluster=testnet) |
| Record guardian heartbeat | 467768027 | [Original receipt](https://explorer.solana.com/tx/2x9NVZGbG14ezqf4ivWLtXSmMfevBMz1GGBJZvdJgKYMhJrRSuebVMZhNFexFPoBfhbZ7CvZNKgX7cxStw8XE6A8?cluster=testnet) |

## Protected accounting

Official JitoSOL purchase and the subsequent atomic DJ01 → IB01 transferred **111,804,320 base units** into protected principal. The actual recorded initial book value, cumulative contributions and HWM all equal **149,999,999 lamports**. Config V2 remains unpaused and the V3 distribution is Idle. Both pending balances, opening yield, settlement counters and KIF liability ended at zero. Guardian slot 0 recorded activity for period 692. Net protocol SOL movement and new KIF liability were both zero.

| Permanent technical SOL basis | Lamports |
| --- | ---: |
| distribution escrow | 329,600 |
| kif sol | 329,600 |
| operational sol | 329,600 |
| pending sol | 329,600 |
| principal sol queue | 329,600 |

The Testnet campaign recorded **7,864,636,400 lamports** gross commitments, including the temporary Buffer deposits and transfers that recycled funds. The gross authorization had **2,135,363,600 lamports** remaining. The opening source debit was **2,688,297,075 lamports**, including its transaction fee. Total network fees were **5,375,000 lamports** over **1,042** finalized original transactions. The **2,492,242,075** Loader refund and the Squads transaction-buffer refunds of **4,338,875** and **2,981,850** lamports are separate from budget headroom. Program and ProgramData retained **422,300** and **2,492,262,675** lamports of technical rent respectively.

The existing Testnet installation at slot **466309305** retains its Config V1 ABI and separate authority. A later finalized control matched the earlier installed Program, ProgramData and multisig. [Historical installation](PIV1_2026_11_19_TESTNET_BUILD.md)

**OPEN.** Public delayed withdrawal and downstream settlement still depend on genuine economic eligibility. M4 remains **OPEN_UNWAIVED**. Genuine Squads D-114 incident recovery remains **OPEN and platform-paused**. Mainnet admission is a separate release decision. [Local genesis and build provenance](PIV1_RENT_V2_CANDIDATE.md)
