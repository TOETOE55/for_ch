use for_ch::for_ch;

fn main() {
    for_ch! {
        for x in 0..5;
        for y in 0..x;
        if let Some(z) = Some(2);
        if x - y < z;
        println!("{:?}", (x, y));
    }
}
