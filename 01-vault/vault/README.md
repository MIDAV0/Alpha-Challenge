# Part a) 

Vulnerability and Attack Explanation

The vulnerability in the old OpenZeppelin implementation of ERC-4626 lies in the rounding mechanism when calculating the number of shares a user receives upon depositing assets into the vault. Specifically, when the deposit is small, the number of shares allocated to the user can be rounded down to zero, effectively causing the user to lose their entire deposit. This issue is exacerbated when the vault's exchange rate (assets per share) is manipulated, making it easier for deposits to result in no shares being issued.

Attack Mechanics

1. Initial Setup by Attacker**: The attacker first deposits a minimal amount of assets (e.g., 1 token) into an empty vault, receiving a small number of shares in return. The attacker then artificially inflates the vault's asset pool by directly donating a large number of tokens (e.g., 100,000 tokens). This shifts the exchange rate significantly, making it very high.
2. Exchange Rate Manipulation**: After the donation, the vault now has a large amount of assets relative to the small number of shares issued. This creates a scenario where any subsequent deposit would result in fewer shares being issued, due to the high exchange rate.
3. User Deposit and Loss**: If a user attempts to deposit assets into the vault after this manipulation, the high exchange rate means their deposit would result in a calculation of shares that rounds down to zero, or a negligible amount. Essentially, the user receives little to no shares in return for their deposit.
4. Attacker's Gain**: As the only significant shareholder, the attacker can then withdraw almost all the assets from the vault, effectively stealing the user's deposit.

Payoff for the Attacker

The attacker benefits by effectively "stealing" the deposits of any subsequent users who deposit assets into the manipulated vault. Since the attacker holds the majority (or all) of the shares after manipulating the exchange rate, they can redeem these shares to withdraw almost all the assets in the vault, including those deposited by unsuspecting users. The size of the attack is only limited by the attacker's ability to front-run the victim and the amount they are willing to initially "donate" to manipulate the vault's state.

This exploit takes advantage of the rounding vulnerability and the ability to manipulate the vault's exchange rate, allowing the attacker to profit from deposits that are effectively made worthless by the manipulated conditions.


# Part b)

OpenZeppelin introduced ERC4626 in v4.7.0 (Jun 30 2022) and added mitigation measure to inflation attack in v4.9.0 (May 23 2023) (https://github.com/OpenZeppelin/openzeppelin-contracts/releases?page=2). This will be the time window for searching the effects of this vulnarubility.


# Part c)

Code:
- Rust
- Solidity

To run the code:
- `cargo run` to run the code

Attack scenario:
1. An attacker back-runs the transaction of an ERC4626 pool creation.
2. The attacker mints for themself one share: deposit(1). Thus, totalAsset()==1, totalSupply()==1.
3. The attacker front-runs the deposit of the victim who wants to deposit 20,000 USDT (20,000.000000).
4. The attacker inflates the denominator right in front of the victim: asset.transfer(20_000e6). Now totalAsset()==20_000e6 + 1, totalSupply()==1.
5. Next, the victim's tx takes place. The victim gets 1 * 20_000e6 / (20_000e6 + 1) == 0 shares. The victim gets zero shares.
6. The attacker burns their share and gets all the money.

POS code functionality:
1. Listen to the mempool for the creation of a new ERC4626 pool.
    - Filter the creation of a new ERC4626 pool by tx call data and 0 destination address.
2. Back-run the pool creation transaction with a deposit.
    - Send a flashbots bundle with a deposit to the new ERC4626 pool.
3. Add new vault address to the set of vaults that are being monitored.
4. Listen to the mempool for the deposit of the victim to one of the monitored vaults.
    - Filter the deposit of the victim by tx function selector (0x2c32e4d4) and monitored vault address.
5. Front-run the deposit of the victim with a token transfer to the vault equivalent to the victim's deposit.
    - Send a flashbots bundle with a token transfer to the vault.
6. Burn the share and withdraw all the assets from the vault.