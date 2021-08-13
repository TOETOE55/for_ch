# for_ch

`for_ch` named "for_each", "for_chain"(or even "4ch"), the crate provides a macro to flatten the nested for-loop and if-let.



## Example

```rust
for_ch! {
    for x in 0..10; 
    for y in x..10; // you can add a label before `for`
    if let Some(z) = foo(x, y).await?;
    if x - y < z { continue; }
    println!("x = {}, y = {}, z = {}", x, y, z);
}
```

would expend to

```rust
for x in 0..10 {
    for y in x..10 {
        if let Some(z) = foo(x, y).await? {
            if x - y < z { continue; }
            println!("x = {}, y = {}, z = {}", x, y, z);
        }
    }
}
```

