extern crate lehmer;

use lehmer::Lehmer;
use crate::permutation::permutation_helper::reduce_permutation;
use crate::permutation::constants;
//note: HARD limit at 12, may need to increase for larger size
//todo: maybe have this in config?
//todo: make usize, u32, i32, etc. consistent, may lead to problems when extending
//todo: make the boundary conditions seamless

const DEBUG_ENABLED: bool = false;

pub fn get_label_for_permutation(permutation: &Vec<u8>) -> u32 {
    if permutation.len() == 0 { return 0; }
    let n = permutation.len() - 1;
    let starting_value = sum_factorial(n as u32);

    starting_value + get_lehmer_code_from_permutation(permutation)
}

pub fn get_all_pure_lehmer_codes_of_size(size: u8) -> Vec<u32> {
    if DEBUG_ENABLED {
        println!("\t[get_all_pure_lehmer_codes_of_size = {}]", size);
    }
    if size == 0 {
        return vec![0];
    }
    let mut list: Vec<u32> = vec![];

    for lehmer_code in 0..constants::FACTORIALS[size as usize] {
        let permutation: Vec<u8> =
            get_permutation_from_lehmer_code(size as usize, lehmer_code as usize);
        let reduced_permutation = reduce_permutation(&permutation);
        if DEBUG_ENABLED {
            println!("\treducing permutation: {:?}\
            \n\treduced permutation: {:?}",
            permutation, reduced_permutation);
        }
        if permutation.len() == reduced_permutation.len() {
            list.push(lehmer_code as u32);
        }
    }

    if DEBUG_ENABLED {
        println!("\tList: {:?}", list);
    }

    return list;
}

pub fn get_lehmer_code_from_permutation(permutation: &Vec<u8>) -> u32 {
    Lehmer::from_permutation(&*permutation).to_decimal() as u32
}

pub fn get_permutation_from_label(label: u32) -> Vec<u8> {
    if DEBUG_ENABLED { println!("get_permutation_from_label : {}", label) }
    //todo: make these boundary conditions seamless
    if label == 0 {
        return vec![];
    }
    let mut n: usize = 0;
    for size in 0..constants::SUM_FACTORIALS.len() {
        if label < constants::SUM_FACTORIALS[size] {
            n = size;
            break;
        }
    }

    if DEBUG_ENABLED { println!("size_of_permutation : {}", n) }

    //todo: fix bug here
    let starting_code = constants::SUM_FACTORIALS[n - 1];
    let lehmer_code = (label - starting_code) as usize;
    get_permutation_from_lehmer_code(n, lehmer_code)
}

pub fn get_permutation_from_lehmer_code(n: usize, lehmer_code: usize) -> Vec<u8> {
    Lehmer::from_decimal(lehmer_code, n).to_permutation()
}

fn factorial(n: u32) -> u32 {
    if n == 0 { return 1; }
    n * factorial(n - 1)
}

//todo: make more efficient [memoization, SUM_FACTORIALS]
pub fn sum_factorial(n: u32) -> u32 {
    return constants::SUM_FACTORIALS[n as usize];
}

//todo: make linear
fn get_personal_lehmer_from_permutation(permutation: &Vec<u8>) -> u32 {
    let mut lehmer_code = 0;
    let n = permutation.len();
    for pos in 0..n {
        let mut ct_of_smaller_to_left = 0;
        for pos_2 in 0..pos {
            if permutation[pos_2] < permutation[pos] {ct_of_smaller_to_left += 1;}
        }
        lehmer_code += constants::FACTORIALS[n-pos-1]*
            ((permutation[pos as usize] - ct_of_smaller_to_left) as u32);
    }
    return lehmer_code;
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
    let input_permutations: Vec<Vec<u8>> =
        vec![
            vec![],
            vec![0],
            vec![0, 1], vec![1, 0],
            vec![0, 1, 2], vec![0, 2, 1], vec![1, 0, 2], vec![1, 2, 0], vec![2, 0, 1], vec![2, 1, 0]
        ];

    let expected_output =
        vec![
            0,
            1,
            2, 3,
            4, 5, 6, 7, 8, 9
        ];

    for pos in 0..input_permutations.len() {
        let lehmer_code = get_label_for_permutation(
            &input_permutations[pos as usize]);
        println!("For Permutation: {:?}, Lehmer Code : {:?}", input_permutations[pos], lehmer_code);
        assert_eq!(lehmer_code, expected_output[pos]);
    }
}

#[test]
fn get_precompute_data() {
    print!("[");
    for size in 0..13 {
        print!("{}, ", sum_factorial(size));
    }
    print!("]");
}

#[test]
fn test_get_permutation_from_label() {
    let labels =
        vec![
            0,
            1,
            2, 3,
            4, 5, 6, 7, 8, 9
        ];

    let expected_output: Vec<Vec<u8>> =
        vec![
            vec![],
            vec![0],
            vec![0, 1], vec![1, 0],
            vec![0, 1, 2], vec![0, 2, 1], vec![1, 0, 2], vec![1, 2, 0], vec![2, 0, 1], vec![2, 1, 0]
        ];


    for pos in 0..labels.len() {
        let permutation = get_permutation_from_label(labels[pos as usize].clone());
        println!("For Code: {:?}, Permutation : {:?}", labels[pos], permutation);
        assert_eq!(permutation, expected_output[pos]);
    }
}

#[test]
fn test_get_all_pure_lehmer_codes_of_size() {
    for size in 0..5 {
        let lehmer_codes = get_all_pure_lehmer_codes_of_size(size);
        let mut permutations: Vec<Vec<u8>> = vec![];
        for code in &lehmer_codes {
            permutations.push(
                get_permutation_from_lehmer_code(
                    size as usize, *code as usize));
        }
        if DEBUG_ENABLED {
            println!("size: {}, lehmer_codes: {:?}, permutations: {:?}", size, &lehmer_codes, permutations);
        }
    }
}

#[test]
fn test_lehmer_code_generation() {
    let permutations: Vec<Vec<u8>> = vec![
        vec![0, 1, 2, 3, 4],
        vec![4, 3, 2, 1, 0],
        vec![3, 2, 4, 1, 0]
        ];

    for permutation in permutations {
        let personal_lehmer = get_personal_lehmer_from_permutation(&permutation);
        let module_lehmer = get_lehmer_code_from_permutation(&permutation);
        println!("Permutation: {:?}\n\
        Personal : {}, Module generated : {}", &permutation, personal_lehmer, module_lehmer);
        assert_eq!(personal_lehmer, module_lehmer);
    }

}