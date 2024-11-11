# DoS unbounded operation
## Description
- Vulnerability Category: `Denial of Service`
- Severity: `Medium`
- Detectors: [`dos-unbounded-operation`](https://github.com/CoinFabrik/scout-soroban/tree/main/detectors/dos-unbounded-operation)
- Test Cases: [`dos-unbounded-operation-3`](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3)

Each block in a Stellar Blockchain has an upper bound on the amount of gas that can be spent, and thus the amount computation that can be done. This is the Block Gas Limit. If the gas spent exceeds this limit, the transaction will fail.

In this smart contract a malicious user may modify the smart contract's conditions so that any transaction coming after will fail, thus imposing a denial of service for other users.

## Exploit Scenario		
In the following example, a contract has a function ´unsafe_loop_with_array´, which contains a for loop that iterates over a range of numbers from 0 to the lenght of the array ´unknown_array´. The issue is that if the length of the array is extremely large, it would cause the loop to execute many times, potentially leading to an unusable state of the contract.

```rust
 pub fn unsafe_loop_with_array(unknown_array: BytesN<8>) -> u32 {
        let mut sum = 0;
        for i in 0..unknown_array.len() {
            sum += i;
        }
        sum
    }
```
The vulnerable code example can be found [here](https://github.com/CoinFabrik/scout-soroban/tree/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3/vulnerable-example).
			
## Remediation
To solve this, instead of relying on an external parameter, we should introduce a known value directly into the loop.

```rust
  pub fn safe_loop_with_array() -> u64 {
        let mut sum = 0;
        let known_array = [0; 8];
        for i in 0..known_array.len() {
            sum += i;
        }
        sum as u64
    }
```
The remediated code example can be found [here](https://github.com/CoinFabrik/scout-soroban/blob/main/test-cases/dos-unbounded-operation/dos-unbounded-operation-3/remediated-example/lib.rs).

## References
- https://consensys.github.io/smart-contract-best-practices/attacks/denial-of-service
- https://consensys.github.io/smart-contract-best-practices/development-recommendations/general/external-calls/
