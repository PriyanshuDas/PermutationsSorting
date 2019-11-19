//use std::time::{Instant};
#[macro_use] extern crate text_io;

mod permutation;
//use permutation::permutations_generator;

fn main() {
    println!("Enter the size of permutations for which to generate: ");
    let n = read!();
    let data =
        permutation::distance_calculator::generate_data_for_size_up_to(n);
    for permutation_size_data in data {
        permutation_size_data.print_summary();
    }
}

//fn generate_permutation(n: _) {
//    let permutations_up_to_size_n =
//        permutations_generator::generate_permutations_up_to_size(n);
//    permutations_generator::print_permutations(permutations_up_to_size_n);
//}