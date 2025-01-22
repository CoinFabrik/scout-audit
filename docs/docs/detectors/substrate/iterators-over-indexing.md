# Iterators over indexing

## Description

- Issue Category: `Arithmetic`
- Issue Severity: `Medium`
- Detectors: [`iterators-over-indexing`](https://github.com/CoinFabrik/scout-audit/blob/main/detectors/substrate-pallets/iterators-over-indexing/src/lib.rs)

It warns if a `for` loop uses indexing instead of an iterator. If the indexing explicitly ends at `.len()`, the lint will not trigger.

### Why is this bad?

Accessing a vector by index is slower than using an iterator. This is because iterators are optimized for sequential access, avoiding repeated bounds-checking at runtime. Also, if the index is out of bounds it will cause the program to panic, potentially leading to runtime errors

### Issue Example

Consider the following substrate pallet:

```rust
#[pallet::call_index(1)]
pub fn set_sum(origin: OriginFor<T>) -> DispatchResult {
    let _sender = ensure_signed(origin)?;

    let mut new_sum = 0_u32;

    if let Some(v) = <Dummy<T>>::get() {
        for i in 0..128 {
            new_sum += v[i];
        }
    }

    <Sum<T>>::mutate(|sum| {
        *sum = Some(new_sum);
    });

    Ok(())
}
```

The provided function contains a for loop that uses indexing to access elements of the v vector.

### Remediated example

Consider using an iterator to iterate over the array

```rust
pub fn set_sum(origin: OriginFor<T>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            let mut new_sum = 0;

            if let Some(v) = <Dummy<T>>::get() {
                for i in v.iter() {
                    new_sum += i;
                }
            }

            <Sum<T>>::mutate(|sum| {
                *sum = Some(new_sum);
            });

            Ok(())
        }
```

## How is it detected?

Find expressions inside the function that call some method that has te name `get` and checks how the for loop is used, if it's used without an iterator, thorws a warning
