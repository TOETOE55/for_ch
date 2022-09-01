use for_ch::*;

fn main() {
    for_ch! {
        for x in 0..10;
        for y in 0..x, for _ in 0..5;
        if let Some(z) = Some(2);
        if x - y < z else { println!("x - y >= 2: {:?}", (x, y)); } ;
        println!("x - y <  2: {:?}", (x, y));
    }
}
