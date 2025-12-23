***

## DePIN Data Publication Mechanisms \& Verification Patterns: Comprehensive Report

### Executive Summary

DePIN networks fundamentally differ from traditional blockchains in their data architecture. Rather than storing transaction histories, they must publish cryptographic commitments to physical infrastructure work (stored data, provided coverage, computed results, network bandwidth) while keeping raw data off-chain. The critical innovation across successful DePINs is the elimination of centralized oracles. Instead, these networks employ distributed verification mechanisms—Merkle-tree proofs, multi-signature witness chains, cryptographic consensus algorithms, and zero-knowledge proofs—that allow the blockchain to cryptographically verify that infrastructure work occurred without trusting a single third party or revealing sensitive data.[^1][^2][^3][^4][^5]

This report details how six major DePIN categories implement on-chain data publication and the verification patterns that enable this architecture.

***

### Storage DePINs: Merkle Commitments \& Cryptographic Proofs

#### **Filecoin: Proof of Replication (PoRep) Architecture**

Filecoin's data publication mechanism represents the gold standard for storage verification. Rather than publishing stored data to the blockchain, Filecoin storage providers publish only a cryptographic commitment: the Merkle root of their replicated dataset plus a SNARK-compressed proof of the sealing process.[^1][^6]

**The Sealing and Commitment Flow:**

Storage providers receive client data in 32GB sectors. The provider generates a unique SEAL key for that sector, then performs a slow sequential encoding process that creates a replica (R) of the original data. This encoding is deliberately computationally expensive—it requires one sequential hash operation per node in the replica, with each hash dependent on the previous one, making it impossible to parallelize. The result is a lattice-like graph structure embedded in the replica.[^7][^1]

The provider then computes the Merkle root (rt) of this encoded replica. This root becomes the cryptographic commitment: a 256-bit hash that mathematically proves possession of the specific replica without revealing the data itself. The provider generates a proof (πSEAL) demonstrating the sealing was performed correctly, then compresses this proof using SNARK (Succinct Non-Interactive Argument of Knowledge) technology. The result: what was originally gigabytes of proof material compresses to approximately 200 bytes.[^6]

Finally, the provider signs this Merkle root with their keypair and submits the complete package to the Filecoin blockchain:

- Merkle root: 32 bytes
- SNARK proof: ~200 bytes
- Provider signature: 64 bytes
- **Total on-chain per sector: ~500 bytes**[^1][^6]

**Verification Without Revealing Data:**

When audited, the Filecoin network issues a random challenge specifying a particular leaf position (Rc) in the Merkle tree of the provider's replica. The provider responds not with the full replica, but with only the Merkle path—the logarithmic number of hashes required to reach the root from that leaf. For a 32GB sector divided into 32-byte leaves (~1 billion leaves), the Merkle path contains only ~30 hashes, or about 960 bytes.[^7][^6][^1]

The blockchain then performs simple verification: hash the provider's response along the claimed path and verify it reaches the published Merkle root. If correct, the provider demonstrated possession of the actual data without transmitting it. If incorrect, the provider either deleted data or is dishonest—either way, rewards are withheld and penalties applied.[^1]

**Key Innovation: PoRep is Proof-of-Storage AND Proof-of-Space**

Unlike simple hash commitments (which could be computed from anywhere), PoRep requires executing the slow sequential encoding algorithm using the original client data. To forge a proof, an attacker would need to re-execute this encoding from scratch using actual client data. This computational barrier makes proof forgery economically infeasible—the cost of forging a proof exceeds the cost of genuinely storing the data.[^6][^7]

**On-Chain vs. Off-Chain Architecture:**

- **On-chain:** Only Merkle commitments and cryptographic proofs (~500 bytes per sector)
- **Off-chain:** Complete replica stored on provider's disk; IPFS or local storage; available for client retrieval
- **Data availability:** Decoupled from blockchain capacity. Petabytes of storage require only kilobytes of on-chain state[^1]

![Filecoin PoRep: From Data Sealing to On-Chain Commitment and Verification](https://ppl-ai-code-interpreter-files.s3.amazonaws.com/web/direct-files/90eb10efd201280b9db9b96e3f5a4d78/b079f63f-f796-47bb-8214-9fe67565d454/403f5fcf.png)

Filecoin PoRep: From Data Sealing to On-Chain Commitment and Verification

***

#### **Arweave: Succinct Proof of Access (SPoA)**

Arweave implements permanent data storage by making new blocks contain cryptographic proof of access to random historical data blocks. This forces miners to maintain the entire blockweave (blockchain + data archive) to continue earning rewards.[^8][^9][^10]

**The Data Publication Pattern:**

New blocks in Arweave must include a Merkle proof referencing a random chunk from historical data. This proof serves as evidence that the miner has that data available. The key innovation: Arweave uses a Verifiable Delay Function (VDF) to generate deterministic but unpredictable challenges that unlock sequential SPoA proofs. Miners cannot predict future challenges and thus cannot selectively maintain only profitable data—they must maintain everything.[^9][^10]

Each proof publication includes:

- Current block hash (256 bits)
- Random historical data chunk reference (~32 bytes)
- VDF commitment for sequential proof generation (~256 bits)
- Merkle proof (variable size, ~1-2KB for typical challenges)
- **Total per block: ~1-2 KB**[^10][^9]

**Three-Phase Proof Verification:**

Challenge: Verifier Bob specifies a random offset into the merkelized dataset using the VDF unlock system.

Proof: Prover Alice retrieves the exact chunk from storage, constructs the Merkle path from that chunk's leaf to the dataset root, and submits both the chunk and the path.

Verification: Bob recursively validates each hash in the path. Critically, early mismatch terminates computation—if any intermediate hash fails verification, Bob immediately rejects the entire proof without computing remaining hashes.[^9][^10]

**Packing Mechanism Prevents Fraud:**

Arweave uses RandomX hashing to pack chunks differently for each miner. This prevents a miner from claiming multiple replicas of the same data (each replica would require different packing), ensuring that claimed storage actually represents unique data committed to the network.[^10][^9]

***

### Wireless/Coverage DePINs: Multi-Signature Witness Chains

#### **Helium: Proof of Coverage with Randomized Witnesses**

Helium's Proof of Coverage demonstrates how distributed physical verification replaces centralized oracles. Rather than a single oracle attesting to a hotspot's location and coverage, the network uses cryptographic multi-signature attestations from independent, geographically distributed witnesses.[^2][^3][^11]

**The Challenge-Response-Witness Flow:**

1. **Challenge Creation:** The network selects a random challenger hotspot and assigns it a target hotspot. The challenger generates an ephemeral keypair (used only for this challenge) and creates a PoC challenge packet encrypted with the target hotspot's public key.[^3]
2. **Broadcast \& Witness:** The challenger broadcasts the encrypted challenge across the Helium P2P network. Any hotspot in RF range can receive it—these become potential witnesses.[^3]
3. **Target Response:** The target hotspot decrypts the challenge using its private key (proving it has the private key corresponding to its claimed identity) and immediately broadcasts the decrypted challenge back across the network.[^3]
4. **Witness Signing:** Between 10 and 25 witness hotspots that observed both the target's response and the challenger's original broadcast cryptographically sign their observations with their own private keys.[^2][^3]
5. **On-Chain Publication:** All signatures are submitted to the blockchain:
    - Challenge ID (ephemeral key SHA256): 32 bytes
    - Target hotspot signature: 64 bytes
    - 10-25 witness signatures: 640-1,600 bytes
    - Metadata (signal strength, timestamps): 100-300 bytes
    - **Total per PoC: ~1-2 KB**[^2][^3]

**Why This Design Prevents Fraud:**

To forge a Proof of Coverage claim without legitimate coverage:

- Attacker must compromise the target hotspot's private key OR substitute their own hotspot AND
- Attacker must compromise 10-25 independent witness hotspots' private keys AND
- Attacker must defeat the random witness selection algorithm (cannot predict which hotspots will witness)

The attack cost—obtaining multiple legitimate hotspots in geographic proximity, securing their private keys, and coordinating them to sign false proofs—far exceeds the reward value. Physical location becomes verifiable through cryptography rather than GPS spoofing or oracle attestation.[^2][^3]

**Evolution to Fairness:**

Earlier versions used a "first 25 witnesses wins" approach, creating a race condition where fast-responding witnesses could capture disproportionate rewards. Current implementations wait 20 blocks, randomly shuffle received witnesses, and select 10-25, ensuring geographic diversity and fairness.[^2]

***

### Compute DePINs: Distributed Consensus Without Oracles

#### **Bittensor: Weight Vectors \& Stake-Weighted Yuma Consensus**

Bittensor demonstrates how distributed consensus algorithms can aggregate work quality assessments without centralized oracle judgment. The system works in phases: off-chain validation, on-chain consensus, and automated emission distribution.[^12][^13][^14][^15]

**The Distributed Validation Process:**

In each subnet, 300+ validators independently run benchmark tasks on all miners. Each validator computes performance scores (on a 0-65535 scale) reflecting how well each miner solved the designated problem. These computations happen entirely off-chain—the blockchain never sees raw performance data.[^13][^14][^15]

Each validator then normalizes these raw scores into a weight vector: [w₁, w₂, w₃, ... w_n] where:

- Each weight represents the validator's assessment of that miner's relative performance
- All weights sum to exactly 1.0 (normalized probability distribution)
- The weight directly determines what fraction of mining emissions that miner receives[^14][^13]

The validator signs this weight vector with its keypair and submits to the blockchain:

- Weight vector: 1,000 miners × 2 bytes = 2,000 bytes
- Validator signature: 64 bytes
- Validator's current stake: 8 bytes
- **Per validator submission: ~2.2 KB**[^15]

With 300 validators per subnet, each epoch accumulates ~660 KB of on-chain weight data. However, the blockchain performs the critical function: **consensus aggregation**.[^13][^15]

**The Yuma Consensus Algorithm: Oracle Replacement**

Rather than a single oracle declaring which miners are best, the blockchain algorithm reaches consensus through stake-weighted voting:

1. **Stake-Weighted Median:** Validators are sorted by their delegated stake. The algorithm computes a cumulative stake and selects the median validator—the one where cumulative stake reaches 50%.
2. **Consensus Weights:** That median validator's weight vector becomes the "consensus baseline" for the epoch.
3. **Clipping Mechanism:** For each miner position, if another validator's weight diverges more than 1.5x from the consensus median, that outlier weight is clipped (capped) at 1.5x median. This prevents colluding validators with large stakes from pushing weights toward specific miners.
4. **Emission Distribution:** The clipped consensus weights determine how the 41% of subnet emissions are split among miners. Miners scoring well across multiple independent validators receive larger allocations.[^12][^14][^13]

**Why Yuma Consensus Outperforms Oracles:**

- **Distributed:** 300+ validators, not 1 oracle
- **Stake-weighted:** Larger stakes have more influence but attacking requires controlling 51%+ stake (~millions of TAO)
- **Penalty for collusion:** Validators whose scores deviate from consensus are clipped (their influence reduced), financially penalizing coordinated attacks
- **Transparent:** On-chain algorithm is auditable and trustless
- **Scalable:** Works across 32+ subnets with independent scoring mechanisms

**Proof of Weights: Preventing Weight Copying**

New innovation to prevent validators from copying weights from other validators (saving computational work while still earning rewards):

- Validators must provide cryptographic witness proof of actual mining queries performed
- Proof submitted alongside weights must demonstrate mathematical consistency: If I ran these queries and got these results, my weights must follow logically
- On-chain verification of witness-to-weight consistency
- Failure to provide valid proof results in 40% dividend penalty[^12]

![Bittensor Yuma Consensus: From Distributed Scoring to On-Chain Aggregate Consensus](https://ppl-ai-code-interpreter-files.s3.amazonaws.com/web/direct-files/90eb10efd201280b9db9b96e3f5a4d78/a64471b4-d053-45b7-9e1a-a5ceef755663/5391d65a.png)

Bittensor Yuma Consensus: From Distributed Scoring to On-Chain Aggregate Consensus

***

### Data DePINs: Zero-Knowledge Proofs \& Rollup Architecture

#### **Grass Network: ZK-SNARK Batching for Privacy-Preserving Verification**

Grass solves a fundamental DePIN challenge: How to prove data was collected (for payment), while preserving privacy and maintaining scalability for millions of daily requests. The solution combines zero-knowledge proofs with batch processing via Solana rollups.[^4][^16][^17]

**The Data Collection and Proof Generation Flow:**

1. **Encrypted Request Forwarding:** An AI company requests web data through Grass. Their request is encrypted as a data packet with routing information only (not request content). The Grass Node forwards it through the P2P network, with each hop adding a digital signature to prove the packet was transmitted.[^17][^4]
2. **Response Collection:** The website returns data encrypted via ECDH (Elliptic-Curve Diffie-Hellman), creating a secure session between the node and the website. Only the node can decrypt it.
3. **Session Validation:** The Validator receives all session records: session key, source node IP, target website URL, timestamp, encrypted response data. The validator also verifies all digital signatures from all parties involved.[^16][^4]
4. **Individual ZK Proof Generation:** For each session, the validator generates a zero-knowledge SNARK proof that proves:
    - Session was encrypted correctly
    - All parties signed authentically (without revealing signatures)
    - The request was routed through the declared node
    - Response was received at the declared timestamp
    - **But reveals nothing about:** the actual web content, the user's IP, or the specific website accessed[^4][^16]

Each ZK proof compresses to approximately 256 bytes.[^4]
5. **Batch Processing:** The ZK Processor collects hundreds or thousands of individual session proofs. It generates a single aggregate ZK proof verifying that all constituent proofs are valid. This rollup proof compresses 10,000 individual session proofs (each 256 bytes = 2.56 MB raw) into a single 1 KB proof.[^17][^4]
6. **On-Chain Settlement:** The 1 KB rollup proof plus metadata is submitted to Solana L1:
    - Rollup ZK proof: 1 KB
    - Session metadata (URLs, timestamps, node IDs): ~100 bytes per session
    - **For 10,000 sessions: ~1.05 KB + metadata**[^17]

**Compression and Privacy Achieved:**

Traditional oracle approach: Validator would need to see all web data, publish data sources on-chain (privacy violation), and validate each transaction individually.

Grass approach:

- **Privacy:** No raw web data visible to validators or blockchain; only ZK proofs confirm proper handling
- **Compression:** 10,000 sessions reducing from 2.56 MB to 1 KB on-chain = 2,560x compression
- **Immutability:** Metadata on-chain proves data origin; actual content linked in Grass Data Ledger with proof references
- **Scalability:** Can handle tens of millions of web requests per minute[^16][^4][^17]

**Key Components:**

- **Routers:** Maintain connection between validators and nodes; ensure network integrity
- **Validators:** Verify transactions; generate ZK-SNARK proofs; batch session data
- **ZK Processor:** Aggregates individual proofs into single rollup proof; submits to Solana
- **Grass Data Ledger:** Immutable off-chain storage of collected datasets with proof linkage

![Grass Network: Zero-Knowledge Proof Batching and Rollup Architecture](https://ppl-ai-code-interpreter-files.s3.amazonaws.com/web/direct-files/90eb10efd201280b9db9b96e3f5a4d78/5626c38d-aea1-48fb-8b3f-e34047879bcb/a4175864.png)

Grass Network: Zero-Knowledge Proof Batching and Rollup Architecture

***

### Video/Content DePINs: Multi-Stage Merkle Proofs

#### **AIOZ Network: Proof of Transcoding, Storage, and Delivery**

AIOZ implements a three-stage verification system where different entities verify different aspects of work, eliminating the need for a single oracle to judge the entire process.[^5][^18]

**Stage 1: Proof of Transcoding (PoT)**

Content Owners upload video segments to Edge Nodes for processing (e.g., encoding 4K video into 1080p, adaptive bitrate, etc.). The Edge Nodes return transcoded segments. The Content Owner must verify transcoding quality before payment.[^18][^5]

Verification: Extract N frames from original video V₁ and N frames from transcoded video V₂. Compare frame hashes—do the transcoded segments match the original content? Merkle root verification shows content consistency without comparing every frame:

- Content Owner computes TOP HASH (Merkle root) of original
- Receives transcoded segments from Edge Node
- Compares Merkle roots of original vs. transcoded
- If Content Owner detects quality flaws, they submit disputed segment + proof to HUB Node[^5][^18]

**Stage 2: Proof of Storage (PoS)**

HUB Nodes (infrastructure coordinators) challenge Edge Nodes to prove they're actually storing video. Random challenge: "Provide the hash at block index X, location Y in your Merkle tree".[^18][^5]

Edge Node responds with:

- Requested hash (64 bytes)
- Merkle path from that hash to TOP HASH (variable, ~1KB)
- Timestamp of response

HUB Node verifies:

- Hash reaches TOP HASH through declared path
- Response submitted within time limit (proving node has quick access to data)
- If timeout or wrong hash: Contract cancelled, Edge Node penalized[^5][^18]

**Stage 3: Proof of Delivery (PoD)**

Final verification that viewers actually received content:

- Viewer sends request + digital signature to Edge Node
- Edge Node delivers content
- Edge Node collects viewer's signature as payment proof
- Edge Node reports PoD to HUB with viewer signature
- HUB verifies signature authenticity (proves viewer participated)
- Payment automatically transferred[^18][^5]

**On-Chain Data Publication:**

- Normal case (no disputes): Only settlement records published = minimal on-chain data
- Dispute case: Merkle proofs + contested hashes published = ~1KB per disputed segment
- **Average per delivery: 0 bytes (settlement is deterministic)**

This staged verification prevents any single entity from controlling the system. Content Owner verifies quality, HUB verifies storage, Viewer verifies delivery, and blockchain only needs to arbitrate disputes.[^5][^18]

***

### Comparative Verification Patterns

![DePIN Verification Patterns: Comprehensive Cross-Category Comparison](https://ppl-ai-code-interpreter-files.s3.amazonaws.com/web/direct-files/90eb10efd201280b9db9b96e3f5a4d78/abf7dbac-98bd-4027-8710-43b4a93c1da8/5a08be66.png)

DePIN Verification Patterns: Comprehensive Cross-Category Comparison

***

### Why Distributed Verification Replaces Centralized Oracles

DePINs avoid centralized oracle architectures for fundamental reasons:

**1. Inherent Verifiability of Physical Work**
Physical infrastructure proofs are cryptographically self-verifying. A storage node either has data or doesn't—no oracle needed to judge. A hotspot either received RF transmissions or didn't—witnesses can attest directly. Compute results are deterministic—multiple validators independently scoring miners can reach consensus.[^1][^2][^13]

**2. Economic Incentive Alignment**
Nodes and validators have direct financial incentives to report truthfully:

- False reports forfeit current epoch rewards
- Slashing penalties destroy stake
- Long-term reputation loss (validators downgraded)

Oracles lack this direct incentive structure—they earn the same fee whether their data is accurate or not, creating weak truthfulness guarantees.[^6][^13][^9]

**3. Distributed Verification Capacity**
Unlike price feeds (which require trusted data sources), infrastructure verification can be distributed:

- Any hotspot can witness any PoC challenge
- Any node can run Merkle path verification
- Any validator can independently score miners
- Multiple witnesses provide Byzantine-fault tolerance

This distribution makes consensus attacks exponentially more costly.[^2][^3][^13]

**4. Privacy Preservation Through Cryptography**
Zero-knowledge proofs (Grass, Arweave) and Merkle commitments (Filecoin) prove facts without revealing underlying data. Traditional oracles would require exposing all data to a third party—a fundamental privacy violation. Cryptographic proofs enable verification while maintaining confidentiality.[^9][^17][^1]

**5. Scalability Through Aggregation**
Distributed verification enables scalability that centralized oracles cannot achieve:

- Grass batches 10,000 sessions into 1 KB rollup
- Bittensor aggregates 1,000 miner scores with consensus algorithm
- AIOZ uses staged verification (different verifiers handle different stages)
- Filecoin uses logarithmic Merkle path verification (30 hashes for 1 billion leaves)

Central oracles must process every transaction individually, creating bottlenecks.[^13][^5][^17][^1]

***

### Data Architecture Best Practices Across DePINs

**1. Commit-Only On-Chain**
Publish cryptographic commitments (Merkle roots, ZK proof hashes, signatures), not raw data. This achieves:

- Minimal on-chain footprint (Filecoin: 500 bytes per 32GB)
- Full auditability (can verify commitment against off-chain data)
- Immutability (commitments can't be retroactively changed)
- Privacy (raw data never exposed)

**2. Logarithmic Verification Complexity**
Use Merkle trees to enable O(log n) verification:

- Challenge specifies random leaf position
- Prover returns only path to root (32 hashes max for 1 billion leaves)
- Verifier confirms path reaches known root
- Works for any dataset size without transmission burden

**3. Multi-Signature Distributed Attestation**
Require multiple independent, geographically distributed parties to sign attestations (Helium PoC model):

- Single hotspot fraud impossible
- Would need to compromise multiple hotspots
- Random witness selection prevents prediction
- Cost of attack exceeds reward value

**4. Batch Processing \& Rollups**
Accumulate individual proofs, generate aggregate proof:

- Grass: 10,000 session proofs → 1 KB rollup (2,560x compression)
- Solana rollups: Compress multiple transactions into single proof
- Enable millions of transactions to settle via single on-chain proof

**5. Staged Verification**
Different entities verify different stages rather than single oracle verifying everything:

- AIOZ: Content Owner (quality) → HUB (storage) → Viewer (delivery)
- Reduces individual verification burden
- Creates system resilience (failure at one stage doesn't break entire system)
- Enables economic incentives alignment

**6. Proof of Work Mechanisms**
Require computationally expensive proof generation to prevent forgery:

- Filecoin PoRep: Slow sequential encoding (can't parallelize)
- Arweave SPoA: VDF challenges force linear time consumption
- io.net zkGPU: GPU-intensive proof generation
- Makes dishonest proof generation more expensive than honest work

***

### Conclusion

DePIN networks have evolved sophisticated alternatives to centralized oracles by combining cryptographic proofs, distributed consensus, and economic incentives. Merkle commitments enable storage verification without data transmission. Multi-signature witnesses provide physical verification through geographically distributed attestation. Consensus algorithms replace oracle judgment with stake-weighted voting. Zero-knowledge proofs maintain privacy while preserving verifiability.

The result is a data architecture where on-chain state remains minimal (~500 bytes to ~1KB per transaction) while enabling auditable verification of petabytes of off-chain data. This scalability—combined with Byzantine-fault tolerance, privacy preservation, and resistance to economic attacks—creates more robust and trustworthy infrastructure than any centralized oracle model could achieve.[^1][^2][^13][^4][^9]

***

Filecoin Whitepaper - Filecoin: A Decentralized Storage Network[^1]
Helium PoC Update: Randomized Witnesses[^2]
Helium Proof of Coverage Documentation[^3]
Proof of Replication Academic Paper[^7]
Helium Protocol Report 2023 Q1[^11]
What Sets It Apart: Filecoin's Proof System[^6]
Proof-of-Weights - Subnet 2 Documentation[^12]
Consensus-based Weights - Bittensor Blog[^13]
Weight Setting - Templar Documentation[^14]
Grass: A Decentralized Data Network for AI[^4]
AIOZ Network CDN Whitepaper[^5]
Grass Network: How to Gain Benefits[^16]
The Permanent Information Storage Protocol - Arweave[^9]
AIOZ Network - Proof of Assets[^18]
Arweave Lightpaper - The Permanent Information Storage Protocol[^10]
<span style="display:none">[^19][^20][^21][^22][^23][^24][^25][^26][^27][^28][^29][^30]</span>

<div align="center">⁂</div>

[^1]: https://filecoin.io/filecoin.pdf

[^2]: https://www.reddit.com/r/HeliumNetwork/comments/psgqn5/poc_update_randomized_witnesses/

[^3]: https://github.com/novalabsxyz/devdocs/blob/master/blockchain/proof-of-coverage.md

[^4]: https://www.panewslab.com/en/articles/2wiq6zuk7uro

[^5]: https://api-new.whitepaper.io/documents/pdf?id=H14svciHd

[^6]: https://filecoin.io/blog/posts/what-sets-us-apart-filecoin-s-proof-system/

[^7]: https://filecoin.io/proof-of-replication.pdf

[^8]: https://academy.swissborg.com/en/learn/arweave

[^9]: https://www.arweave.org/files/arweave-lightpaper.pdf

[^10]: https://arweave.org/files/arweave-lightpaper.pdf

[^11]: https://www.helium.foundation/protocol-report-2023-q1

[^12]: https://docs.omron.ai/proof-of-weights

[^13]: https://blog.bittensor.com/consensus-based-weights-1c5bbb4e029b

[^14]: https://docs.tplr.ai/validators/weight-setting/

[^15]: https://docs.learnbittensor.org/validators

[^16]: https://nftevening.com/what-is-grass-crypto/

[^17]: https://www.gate.com/learn/articles/grass-a-decentralized-data-network-for-ai/5009

[^18]: https://aioz.network/210121_AIOZ_Whitepaper.pdf

[^19]: https://www.chaintech.network/blog/how-does-render-work/

[^20]: https://web3edge.io/research/what-is-filecoin/

[^21]: https://academy.youngplatform.com/en/cryptocurrencies/rndr-render-network-what-is-how-it-works/

[^22]: https://www.binance.com/en/square/post/22592724969585

[^23]: https://github.com/orgs/akash-network/discussions/614

[^24]: https://static.storj.io/storjv2.pdf

[^25]: https://akash.network/blog/achieving-decentralized-physical-state-consensus-with-witness-chain-on-akash/

[^26]: https://onlinelibrary.wiley.com/doi/10.1155/2022/6998046

[^27]: https://akash.network/blog/akash-network-validator-rewards/

[^28]: https://ira.lib.polyu.edu.hk/bitstream/10397/99821/1/Au_Towards_Practical_Auditing.pdf

[^29]: https://aioz.network/blog/aioz-network-proof-of-assets

[^30]: https://chainplay.gg/blog/what-is-grass-crypto-extension-and-how-to-get-airdrop-stage-2/

