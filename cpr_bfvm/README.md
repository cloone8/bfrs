# cpr_bfvm

[![crate](https://img.shields.io/crates/v/cpr_bfvm.svg)](https://crates.io/crates/cpr_bfvm)

A cross-platform Brainfuck interpreter command line utility, built on [cpr_bf].

For the complete explanation of each possible option, run the `cpr_bfvm` with `--help`.

## Examples

Run the Brainfuck program contained in `helloworld.bf`:

```bash
$ cpr_bfvm helloworld.bf
```

The above example, but with memory cells of 64 bits:

```bash
$ cpr_bfvm helloworld.bf --cellsize u64
```

[cpr_bf]: https://github.com/cloone8/cpr_brainfuck/tree/master/cpr_bf
