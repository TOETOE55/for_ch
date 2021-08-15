# for_ch

`for_ch` named "for_each", "for_chain"(or even "4ch"), the crate provides a macro to flatten the nested for-loop and if-let.



## Example

```rust
for_ch! {
    for x in 0..10;                         // forall x in 0..10,
    // you can add a label before `for`
    for y in x..10, for _ in 0..5;          // forall y in x..x+5, 
    // zipping
    if let Some(z) = foo(x, y).await?;      // exists z. Some(z) = foo(x, y).await?
    // if let guard
    if x - y < z;                           // satisfies x - y < z
    // guard
    println!("x = {}, y = {}, z = {}", x, y, z);
}
```

would expend to

```rust
for x in 0..10 {
    for y in (x..10).zip(0..5) {
        if let Some(z) = foo(x, y).await? {
            if x - y < z {
                println!("x = {}, y = {}, z = {}", x, y, z);
            }
        }
    }
}
```

