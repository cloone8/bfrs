mod cli_args;

use cpr_bf::{BrainfuckVM, DynamicAllocator};

fn main() {
    let hello_world = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    let mut vm = cpr_bf::VMBuilder::new()
        .with_cell_type::<u64>()
        .with_allocator::<DynamicAllocator>()
        .with_preallocated_cells(16)
        .build();

    vm.run_string(hello_world).unwrap();

    // bfrs::run_string::<u16>(hello_world).unwrap();

    // bfrs::run_string::<u32>(hello_world).unwrap();

    // bfrs::run_string::<u64>(hello_world).unwrap();

    // bfrs::run_string::<u128>(hello_world).unwrap();
}
