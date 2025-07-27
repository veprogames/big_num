# big_num

A Rust implementation for Numbers from ± 10 ^ i64::MIN - 9.999.. * 10 ^ i64::MAX. The Primary Use of this is for Incremental Games.

State: usable, but small stuff is missing like serde support

## Features

### Mathematically correct (as much as I know)

- Explicit Variants for Zero, Infinities and NaN
- Comparisons: Infinity != Infinity, NaN != <Anything>
- Most if not all edge cases are handled: For example, what is Infinity * -Infinity again?

### Speed

- Unnormalizing Methods are exposed, allowing you to squeeze out more speed if needed and if you know what you are doing. I might add benchmarks here later
- add, sub, mul, div and some other methods are implemented mutable by default to reduce allocations.
