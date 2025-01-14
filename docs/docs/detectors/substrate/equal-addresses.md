# Equal addresses

## Description

- Category: `Error Handling`
- Severity: `Minor`
- Detectors: equal-addresses
- Test Cases: equal-addresses-1 equal-addresses-2

Functions that receive two addresses as parameters should include a check to ensure they are not the same. This ensures that the addresses represent distinct entities within the system

## Why is it bad?

Failing to verify that two input addresses are distinct can lead to unexpected behavior in smart contracts. For instance, when performing actions such as transferring tokens back to itself, signing permissions, or granting approvals between two addresses, allowing identical addresses can cause erroneous or redundant behavior and a waste of gas.

## Issue example

Consider the following substrate pallet:

```rust
pub fn check_balance(
origin: OriginFor<T>,
from: T::AccountId,
) -> DispatchResult {
let origin = ensure_signed(origin)?;

let user_balance = Self::balance_of(&origin);
let sender_balance = Self::balance_of(&from);

            ensure!(sender_balance >= user_balance, "Insufficient balance");

            Ok(())
        }
```

This function takes two addresses as parameters and checks their respective balances. If the two addresses are identical, the function ends up checking the balance of the same accounts, resulting in unnecessary gas consumption

## Remediated example

Consider checking if the addresses are the same:

```rust
pub fn check_balance(origin: OriginFor<T>, from: T::AccountId) -> DispatchResult {
let origin = ensure_signed(origin)?;

            if from == origin {
                return Err(Error::<T>::SameAddresses.into());
            }
            let user_balance = Self::balance_of(&origin);
            let sender_balance = Self::balance_of(&from);

            ensure!(sender_balance >= user_balance, "Insufficient balance");

            Ok(())
        }
```

## How is it detected?

First, it checks if the function has two or more parameters that are addresses and stores them in a vector. Then, it constructs a control flow graph using the MIR (Mid-level Intermediate Representation) of Dylint and LateLintPass. It checks whether there is a boolean condition within the function that compares the two parameters. If such a check exists, the corresponding elements are removed from the vector, and the process moves to the next function. If no check is found, throws a warning.
