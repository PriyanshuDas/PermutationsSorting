use permutohedron::LexicalPermutation;
use std::time::Instant;

//todo: extract to utilities
pub fn print_3d_vector(permutations_lists: Vec<Vec<Vec<u32>>>) {
    for permutations_list in permutations_lists {
        print_2d_vector(permutations_list);
        println!();
    }
}

pub fn print_2d_vector(permutations_list: Vec<Vec<u32>>) {
    for permutation in permutations_list {
        println!("{:?} ", permutation);
    }
}

pub fn generate_permutations_up_to_size(n: u8) -> Vec<Vec<Vec<u8>>> {
    let mut list_of_permutations = vec![];
    for permutation_size in 0..n {
        let permutations = generate_permutations(permutation_size+1);
        list_of_permutations.push(permutations);
    }
    return list_of_permutations;
}

fn generate_permutations(n:u8) -> Vec<Vec<u8>> {
    let mut permutation = vec![0; n as usize];
    for value in 0..n {
        permutation[value as usize] = value;
    }
    let mut permutations = Vec::new();
    loop {
        permutations.push(permutation.to_vec());
        if !permutation.next_permutation() {
            break;
        }
    }
    return permutations;
}

#[test]
pub fn test_permutation_generation() {
    let mut permutation = vec![1,2,3,4];
    let mut permutations = Vec::new();

    loop {
        permutations.push(permutation.to_vec());
        if !permutation.next_permutation() {
            break;
        }
    }
    assert_eq!(24, permutations.len());
    for permutation in permutations {
        println!("{:?}", permutation);
    }
}

#[test]
pub fn test_permutations_list_generation() {
    let now = Instant::now();
    for size in 0..8 {
        let permutations_up_to_size_n = generate_permutations_up_to_size(size);
        let elapsed_duration = now.elapsed();
        println!("For size {}, time taken (ms) = {:?}, (s) = {:?}",
                 size, elapsed_duration.as_millis(),  elapsed_duration.as_secs());
//        print_permutations(permutations_up_to_size_n);
    }
}
