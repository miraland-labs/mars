# Mars

Derived from and credit to [Ore](https://github.com/HardhatChad/ore), **Mars is a cryptocurrency for sovereign individuals living in Mirascape Horizon across the galaxy.** You can mine Mars from anywhere on Earth. The ultimate goal for Mars is to circulate it between sovereign individuals and intelligent species such as AI, and to exchange it between intelligent species. It uses a novel proof-of-work algorithm to guarantee no miner can ever be starved out from earning rewards. 


## How it works

The primary characterstics of Mars is to offer non-exclusive mining rewards. This means one miner finding a valid solution does not prevent another miner from finding one as well. Rather than setting up every miner in a winner-take-all competition against one another, Mars gives each miner a personalized computational challenge. As long as a miner provides a valid solution to their own individual challenge, the protocol guarantees they will earn a piece of the supply. Since no miner can be censored from the network and valid solutions are non-exclusive, starvation is avoided.


## Supply

Mars is designed to protect holders from runaway supply inflation. Regardless of how many miners are active in the world, supply growth is strictly bounded to a rate of `0 ≤ R(eward) ≤ 10 MARS/min`. In other words, linear. The mining reward rate – amount paid out to miners per valid solution – is dynamically adjusted every 60 seconds to maintain an average supply growth of `10 MARS/min`. This level was chosen for its straightforward simplicity, scale agnosticism, and for striking a balance between the extremes of exponential inflation on one hand and stagnant deflation on the other.


## Program
- [`Consts`](src/consts.rs) – Program constants.
- [`Entrypoint`](src/lib.rs) – The program entrypoint.
- [`Errors`](src/error.rs) – Custom program errors.
- [`Idl`](idl/mars_program.json) – Interface for clients, explorers, and programs.
- [`Instruction`](src/instruction.rs) – Declared instructions and arguments.
- [`Loaders`](src/loaders.rs) – Validation logic for loading Solana accounts.


## Instructions
- [`Initialize`](src/processor/initialize.rs) – Initializes the Mars program, creating the bus, mint, and treasury accounts.
- [`Reset`](src/processor/reset.rs) – Resets the program for a new epoch.
- [`Register`](src/processor/register.rs) – Creates a new proof account for a prospective miner.
- [`Mine`](src/processor/mine.rs) – Verifies a hash provided by a miner and issues claimable rewards.
- [`Claim`](src/processor/claim.rs) – Distributes claimable rewards as tokens from the treasury to a miner.
- [`UpdateAdmin`](src/processor/update_admin.rs) – Updates the admin authority.
- [`UpdateDifficulty`](src/processor/update_difficulty.rs) - Updates the hashing difficulty.


## State
 - [`Bus`](src/state/bus.rs) - An account (8 total) which tracks and limits the amount mined rewards each epoch.
 - [`Proof`](src/state/proof.rs) - An account (1 per miner) which tracks a miner's hash, claimable rewards, and lifetime stats.
 - [`Treasury`](src/state/treasury.rs) – A singleton account which manages program-wide variables and authorities.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
