fn main() {
    let hello_world = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    bfrs::run_string::<u8>(hello_world).unwrap();
    bfrs::run_string::<u16>(hello_world).unwrap();
    bfrs::run_string::<u32>(hello_world).unwrap();
    bfrs::run_string::<u64>(hello_world).unwrap();
    bfrs::run_string::<u128>(hello_world).unwrap();
}
