use bf_macros::bf;

fn main() {
    // added a small thing here to test hello world.
    //TODO: FIX STRLIT INP AND ACTUALLY MAKE GAME OF LIFE.
    println!("{}", bf!(<u8, 30000, 0>
            ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>. hello world
            >---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
    ));
}
