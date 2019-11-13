//todo: use properly

pub const SUM_FACTORIALS: [u32; 13] =
    [1, 2, 4, 10, 34, 154, 874, 5914, 46234, 409114, 4037914, 43954714, 522956314];

pub const FACTORIALS: [u32; 13] =
    [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800, 479001600];


pub fn get_sum_factorials(n: usize) -> u32 {SUM_FACTORIALS[n]}
pub fn get_factorial(n: usize) -> u32 {FACTORIALS[n]}