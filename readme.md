# User Stories for Zaals Finance

## **Actors**
1. Node Operator - create vaults
2. Capital Provider - Deposit Capital into vault earns rewards
3. Position Holder - Capital Provider who holds a position NFT
4. Agent - Monitors node performance and can raise slashing requests and deposits rewards
5. Beneficiary - Receives a share of rewards from the vault ( An off-chain entity supplier who provides services to node operators)

## **Initialization**

* As an node-operator, when I initialize a vault with valid configuration, the vault is created and the formation phase begins. ✅
* As an node-operator, when I initialize a vault with invalid shares or min/max cap mismatch, the transaction fails with "Invalid Configuration". ✅
* As an node-operator, when I initialize a vault a new MPL-Core collection is created. ✅

---

## **Capital Formation**

* As a capital provider, when I deposit capital during Formation phase, a Position NFT is minted and my deposit is recorded in the vault. ✅
* As a capital provider, when I try to deposit after Active phase has started, the transaction fails with "Vault Not Accepting Deposits".✅
* As a capital provider, when I withdraw early in Formation phase, my capital is returned minus the fee and my Position NFT is burned. ✅
* As a capital provider, when I try to withdraw early after the Active phase has begun, the transaction fails with "Capital Locked". ✅
* As a capital provider, I can close my position after fundraise period if the vault didn't reach the min_cap, and withdraw my funds. ✅
* As a node operator, I can clsoe the vault after fundraise period if the vault didn't reach the min_cap and its ata is empty. ✅
  
---

## **Active Phase Behavior**

* As a position holder, when rewards are deposited by the reward distributor, my claimable rewards increase proportionally to my stake. ✅
* As a position holder, when I claim rewards, only the accumulated rewards are transferred. ✅
* As a position holder, When I try to Unlock prinicipal, the transaction failes with "Active Phase, Rewards Locked" error. ✅
* As a buyer, when I purchase a listed Position NFT, I become the new owner of the locked position and rewards. ✅
* As a seller, when I try to list a Position NFT I do not own, the transaction fails with "Unauthorized". ✅
* As a Beneficiary, When I claim rewards, my share of rewards is transferred to my wallet. ✅

---

## **Reward Deposit Validation**

* As a reward distributor, when I deposit rewards during the Active phase, rewards are added to the vault. ✅
* As a reward distributor, when I try to deposit rewards using a non-authorized wallet, the transaction fails with "Invalid Reward Distributor". ✅
* As a reward distributor, when I deposit rewards with the wrong token mint, the transaction fails with "Invalid Reward Token". ✅

---

## **Slashing & Dispute Window**

* As an agent, when I raise a slashing request during the Active phase, a dispute window opens and slashing amount is recorded. ✅
* As a agent, when I submit a slashing request exceeds max_slash_bps, the transaction fails with "Slash Amount Exceeds Limit". ✅
* As an agent, when I try to raise a slashing request outside the Active phase, the transaction fails with "Invalid Phase". ✅
* As an agent, when I submit slashing proof before the dispute window expires, the slash amount is approved. ✅
* As an agent, when I fail to submit proof within the dispute window, the vault dismisses the slash request automatically. ✅
* As a node operator, when I continue depositing rewards during the dispute window, deposits succeed. ✅
* As a position holder, when I try to claim rewards during a dispute, the transaction fails with "Vault in Dispute". ✅

---

## **Closure Phase**

* As a position holder, when I withdraw my principal in Closed phase, I receive my pro-rata capital and my Position NFT is burned. ✅
* As a position holder, when I try to withdraw principal before closure, the transaction fails with "Invalid Phase". ✅
* As a node operator, when I close the vault after Active phase. ✅
---


## Program Flow Architecture
![Programs Arch](diagrams/final-arch.png)