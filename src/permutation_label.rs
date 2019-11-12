extern crate lehmer;

use lehmer::Lehmer;

pub fn get_label_for_permutation(permutation: Vec<u8>) -> u32 {
    if permutation.len() == 0 {return 0}
    let n = (permutation.len() - 1);
    let starting_value = sum_factorial(n as u32);
    return  starting_value + Lehmer::from_permutation(&*permutation).to_decimal() as u32;
}

fn factorial(n: u32) -> u32 {
    if n == 0 {return 1;};
    n*factorial(n-1)
}

//todo: make more efficient [memoization, sum_factorials]
fn sum_factorial(n: u32) -> u32 {
    let mut sum = 0;
    for i in 0..=n {
        sum += factorial(i);
    }
    sum
}

#[test]
fn test_factorial() {
    let factorials = [1, 1, 2, 6, 24];
    for i in 0..factorials.len() {
        assert_eq!(factorials[i], factorial(i as u32));
    }
}

#[test]
fn test_sum_factorial() {
    let sum_factorials = [1, 2, 4, 10, 34];
    for i in 0..sum_factorials.len() {
        assert_eq!(sum_factorials[i], sum_factorial(i as u32));
    }
}

#[test]
fn test_lehmer_from_permutation() {
    let input_permutations: Vec<Vec<u8>> = vec![
        vec![],
        vec![0],
        vec![0, 1],
        vec![1, 0],
        vec![0, 1, 2],
        vec![0, 2, 1],
        vec![1, 0, 2],
        vec![1, 2, 0],
        vec![2, 0, 1],
        vec![2, 1, 0]
    ];

    let expected_output = vec![
        0,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9
    ];

    for pos in 0..input_permutations.len() {
        let lehmer_code = get_label_for_permutation(
            input_permutations[pos as usize].clone());
        println!("For Permutation: {:?}, Lehmer Code : {:?}", input_permutations[pos], lehmer_code);
        assert_eq!(lehmer_code, expected_output[pos]);
    }
}