# DoS unexpected revert with vector

## Description 

- Category: `Authorization`
- Severity: `Critical`
- Detector: [`dos-unexpected-revert-with-vector`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unexpected-revert-with-vector)
- Test Cases: [`dos-unexpected-revert-with-vector-1`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-1) [`dos-unexpected-revert-with-vector-2`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-2) 

This issue of DoS through unexpected revert arises when a smart contract does not handle storage size errors correctly, and a user can add an excessive number of entries, leading to an unexpected revert of transactions by other users and a Denial of Service. 

## Why it is bad?

In Soroban smart contracts, a Denial of Service (DoS) issue through unexpected reverts can occur if the contract does not properly manage storage limits or handle errors when the storage capacity is exceeded. If a user adds an excessive number of entries, it could trigger a revert, causing subsequent transactions by other users to fail unexpectedly, leading to a potential Denial of Service.


## Issue example 

The smart contract we developed for his example allows users to vote for one of different candidates. The smart contract contains a struct named `UnexpectedRevert` that stores the total number of votes, a list of candidates, their votes, and whether an account has voted. It also stores information about the most voted candidate and when the vote will end.

Consider the following `Soroban` contract:

```rust
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum URError {
    // Unexpected Revert Error
    AccountAlreadyVoted = 1,
    CandidateAlreadyAdded = 2,
    CandidateDoesntExist = 3,
    Overflow = 4,
    TimestampBeforeCurrentBlock = 5,
    VoteEnded = 6,
}

#[derive(Debug, Clone, PartialEq)]
#[contracttype]
pub struct State {
    total_votes: u64,
    candidates: Vec<Address>,
    votes: Map<Address, u64>,
    already_voted: Map<Address, bool>,
    most_voted_candidate: Address,
    candidate_votes: u64,
    vote_timestamp_end: u64,
}

const STATE: Symbol = symbol_short!("STATE");

#[contract]
pub struct UnexpectedRevert;

#[contractimpl]
impl UnexpectedRevert {
    pub fn init(env: Env, end_timestamp: u64) -> Result<State, URError> {
        if end_timestamp <= env.ledger().timestamp() {
            return Err(URError::TimestampBeforeCurrentBlock);
        }

        let zero_string: String = String::from_str(&env, "00000000000000000000000000000000");
        let zero_addr = Address::from_string(&zero_string); //CHECK
        let state = State {
            total_votes: 0,
            most_voted_candidate: zero_addr,
            candidate_votes: 0,
            candidates: Vec::new(&env),
            already_voted: Map::new(&env),
            votes: Map::new(&env),
            vote_timestamp_end: end_timestamp,
        };

        env.storage().instance().set(&STATE, &state);
        Ok(state)
    }

    pub fn add_candidate(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }
        if state.already_voted.contains_key(caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            state.candidates.push_back(candidate.clone());
            state.votes.set(candidate, 0);
            Ok(())
        }
    }

    pub fn get_votes_for_a_candidate(env: Env, candidate: Address) -> Result<u64, URError> {
        let state = Self::get_state(env.clone());
        state
            .votes
            .get(candidate)
            .ok_or(URError::CandidateDoesntExist)
    }

    pub fn most_voted_candidate_votes(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.candidate_votes
    }

    pub fn most_voted_candidate(env: Env) -> Address {
        let state = Self::get_state(env);
        state.most_voted_candidate
    }

    pub fn get_total_votes(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.total_votes
    }

    pub fn get_total_candidates(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.candidates.len() as u64
    }

    pub fn get_candidate(env: Env, index: u32) -> Result<Address, URError> {
        let state = Self::get_state(env);
        if index < state.candidates.len() {
            Ok(state.candidates.get(index).unwrap())
        } else {
            Err(URError::CandidateDoesntExist)
        }
    }

    pub fn account_has_voted(env: Env, account: Address) -> bool {
        let state = Self::get_state(env);
        state.already_voted.get(account).unwrap_or(false)
    }

    pub fn vote(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        caller.require_auth();
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }

        if state.already_voted.contains_key(caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            state.already_voted.set(caller, true);
            let votes = state
                .votes
                .get(candidate.clone())
                .ok_or(URError::CandidateDoesntExist)?
                .checked_add(1)
                .ok_or(URError::Overflow)?;
            state.votes.set(candidate.clone(), votes);
            state.total_votes.checked_add(1).ok_or(URError::Overflow)?;
            if state.candidate_votes < votes {
                state.candidate_votes = votes;
                state.most_voted_candidate = candidate;
            }
            Ok(())
        }
    }

    pub fn vote_ended(env: Env) -> bool {
        let state = Self::get_state(env.clone());
        state.vote_timestamp_end <= env.ledger().timestamp()
    }

    pub fn get_state(env: Env) -> State {
        env.storage().instance().get(&STATE).unwrap()
    }
}
```
The smart contract has several functions that allow adding a candidate, getting
votes for a specific candidate, getting the account ID of the most voted
candidate, getting the total votes, getting the total number of candidates,
getting a candidate by index, checking if an account has voted, and voting for
a candidate.

In this case, we see that a vector is being used to store the array of candidates for an election.
Notice how the candidates array push operation has no access control or proper storage management in the function `add_candidate()`, which can cause a revert if the array is full.

The code example can be found [here](https://github.com/CoinFabrik/scout-soroban/blob/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-1/vulnerable-example).

## Remediated example

This issue can be addressed in different ways. 

On the one hand, if the amount of candidates is going to be limited, and only authorized users are going to add new candidates, then enforcing this authorization would be a sufficient fix to prevent attackers from filling the array and producing a denial of service attack.

```rust
pub fn add_candidate(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
    let mut state = Self::get_state(env.clone());
     // Require authorization from an admin set at contract initalization.
    state.admin.require_auth(); 
    if Self::vote_ended(env.clone()) {
        return Err(URError::VoteEnded);
    }
    if state.already_voted.contains_key(caller.clone()) {
        return Err(URError::AccountAlreadyVoted); 
    } else {
        state.candidates.push_back(candidate.clone());
        state.votes.set(candidate, 0);
        Ok(())
    }
}
```

Alternatively, if any user should be authorized to add new candidates, a different data structure should be used, one without the limitations of a vector. For example, a dictionary can be implemented in Soroban by defining a struct for the `Candidate`, accessible through a `DataKey` enum like the one we have here. This data structure does not have the storage limitations of vectors, and using it to handle new candidates would prevent the issue.

```rust
 pub fn add_candidate(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
      caller.require_auth();
      let mut state = Self::get_state(env.clone());
      if Self::vote_ended(env.clone()) {
          return Err(URError::VoteEnded);
      }
      if Self::account_has_voted(env.clone(), caller.clone()) {
          return Err(URError::AccountAlreadyVoted); 
      } else {
          // Replace the Vector with a mapping like structure made with a DataKey enum.
          env.storage().instance().set(&DataKey::Candidate(candidate.clone()), &Candidate{votes: 0});
          state.total_candidates += 1; 
          env.storage().instance().set(&DataKey::State, &state);
          Ok(())
      }
}
```

This remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/blob/main/test-cases/dos-unexpected-revert-with-vector/dos-unexpected-revert-with-vector-2/remediated-example).

## How is it detected?

Checks if the contract uses the vec type without using a require auth previously.

## References

- [SWC-113](https://swcregistry.io/docs/SWC-113)
- https://consensys.github.io/smart-contract-best-practices/attacks/denial-of-service/#dos-with-unexpected-revert
- [Ethernaut: King](https://github.com/OpenZeppelin/ethernaut/blob/master/contracts/src/levels/King.sol)
