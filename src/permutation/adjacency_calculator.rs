use crate::permutation::permutation_label::get_permutation_from_label;
use crate::permutation::constants;
use super::permutation_label;

use crate::permutation::permutations_generator;
use std::cmp::max;

const DEBUG_ENABLED: bool = false;

struct AdjacencyCalculator {
    size: usize,
    // [code][item] = mask
    mask_for_pos_with_items_greater_than_item_for_permutation: Vec<Vec<u32>>,
    // [code][pos][0] = Sum(p_i*i!), [code][pos][1] = Sum(p_i*(i+1)!),
    running_sum_of_item_pos_factorial: Vec<Vec<Vec<u64>>>,
    mask_sum_of_bits_position_factorial: Vec<u64>,
}

fn get_mask_for_pos_with_items_lesser_than_item_for_code(size: usize, code: u32) -> Vec<u32> {
    let mut mask_vector = vec![];
    let permutation =
        permutation_label::get_permutation_from_lehmer_code(size, code as usize);

    let mut item_positions = vec![0 as u8; size];
    for pos in 0..permutation.len() {
        item_positions[permutation[pos] as usize] = pos as u8;
    }

    for item in 0..size {
        let mut mask: u32 = 0;
        for lesser in 0..item {
            mask = (mask | (1 << item_positions[lesser]) as u32)
        }
        mask_vector.push(mask);
    }
    mask_vector
}

fn get_masks_for_pos_with_items_lesser_than_item_for_permutation(size: usize) -> Vec<Vec<u32>> {
    let mut vector = vec![];
    //todo: can be parallelized
    for code in 0..constants::FACTORIALS[size] {
        let vector_for_current_code =
            get_mask_for_pos_with_items_lesser_than_item_for_code(size, code);
        vector.push(vector_for_current_code);
    }
    return vector;
}

fn get_running_sum_of_item_pos_factorial_for_all_codes(size: usize) -> Vec<Vec<Vec<u64>>> {
    let mut vector = vec![];
    //todo: can be parallelized
    for code in 0..constants::FACTORIALS[size] {
        let vector_for_current_code =
            get_running_sum_of_item_pos_factorial_for_code(size, code);
        vector.push(vector_for_current_code);
    }
    return vector;
}

fn get_running_sum_of_item_pos_factorial_for_code(size: usize, code: u32) -> Vec<Vec<u64>> {
    let mut vector_for_current_code = vec![];
    let mut sum_0 = 0;
    let mut sum_1 = 0;
    let mut vector_for_i_0 = vec![];
    let mut vector_for_i_1 = vec![];
    let permutation =
        permutation_label::get_permutation_from_lehmer_code(size, code as usize);
    for pos in 0..size {
        sum_0 += (permutation[pos] as u64) * constants::FACTORIALS[size - pos - 1] as u64;
        vector_for_i_0.push(sum_0.clone());
        if pos < size - 1 {
            sum_1 += (permutation[pos] as u64) * constants::FACTORIALS[size - pos - 2] as u64;
            vector_for_i_1.push(sum_1.clone());
        }
    }
    vector_for_current_code.push(vector_for_i_0);
    vector_for_current_code.push(vector_for_i_1);
    vector_for_current_code
}

fn get_position_factorial_for_set_bits_in_mask(size: usize, mask: u32) -> u64 {
    let mut sum = 0;
    for pos in 0..size {
        if (1 << pos) & mask as usize > 0 {
            sum += constants::FACTORIALS[pos] as u64;
        }
    }
    sum
}

fn get_mask_sum_of_bits_position_factorial(size: usize) -> Vec<u64> {
    let mut masks_sum_vector = vec![];
    for mask in 0..(1 << size) {
        let value_of_mask = get_position_factorial_for_set_bits_in_mask(size, mask as u32);
        masks_sum_vector.push(value_of_mask);
    }
    masks_sum_vector
}

impl AdjacencyCalculator {
    pub fn init(size: usize) -> AdjacencyCalculator {
        return AdjacencyCalculator {
            size,
            mask_for_pos_with_items_greater_than_item_for_permutation:
            get_masks_for_pos_with_items_lesser_than_item_for_permutation(size),
            running_sum_of_item_pos_factorial:
            get_running_sum_of_item_pos_factorial_for_all_codes(size),
            mask_sum_of_bits_position_factorial:
            get_mask_sum_of_bits_position_factorial(size),
        };
    }

    //todo: implement
    pub fn get_adjacency_map(&self) {

    }
}


//todo: implement cleanly
//todo: this needs to be optimized!
pub fn get_code_shifting_item_to_new_position(size: usize,
                                              lehmer_code: u32,
                                              original_pos: usize,
                                              new_after_pos: i8,
) -> u32 {
    let permutation = permutation_label::get_permutation_from_lehmer_code(size, lehmer_code as usize);
    let mut new_permutation = vec![];
    if DEBUG_ENABLED {
        println!("get_code_shifting_item_to_new_position:\
        size: {}, code: {}, shift: ({} {})\tpermutation: {:?}",
                 size, lehmer_code, original_pos, new_after_pos, permutation);
    }
    let pos_1 = (new_after_pos + 1) as usize;

    for i in 0..pos_1 {
        new_permutation.push(permutation[i as usize]);
    }
    new_permutation.push(permutation[original_pos]);

    for i in pos_1..original_pos {
        new_permutation.push(permutation[i]);
    }
    for i in original_pos + 1..size {
        new_permutation.push(permutation[i]);
    }
    if DEBUG_ENABLED {
        println!("get_code_shifting_item_to_new_position: {:?}", new_permutation);
    }
    return permutation_label::get_lehmer_code_from_permutation(&new_permutation);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_mask_for_pos_with_items_greater_than_item_for_code_test() {
        struct TestCase {
            input_permutation: Vec<u8>,
            expected_mask_vector: Vec<u32>,
        }

        let test_cases = vec![
            TestCase {
                input_permutation: vec![0, 1, 2, 3],
                expected_mask_vector: vec![0b0000, 0b0001, 0b0011, 0b0111],
            },
            TestCase {
                input_permutation: vec![3, 1, 2, 0],
                expected_mask_vector: vec![0b0000, 0b1000, 0b1010, 0b1110],
            },
            TestCase {
                input_permutation: vec![3, 2, 1, 0],
                expected_mask_vector: vec![0b0000, 0b1000, 0b1100, 0b1110],
            }
        ];

        for case in test_cases {
            let input_code =
                permutation_label::get_lehmer_code_from_permutation(&case.input_permutation);
            let size = (&case).input_permutation.len();
            let returned_mask =
                get_mask_for_pos_with_items_lesser_than_item_for_code(size, input_code);

            assert_eq!(returned_mask, case.expected_mask_vector);
        }
    }

    #[test]
    fn test_get_running_sum_of_item_pos_factorial_for_code() {
        struct TestCase {
            input_permutation: Vec<u8>,
            expected_summation_vector: Vec<Vec<u64>>,
        }

        let test_cases = vec![
            TestCase {
                input_permutation: vec![0, 1, 2, 3],
                expected_summation_vector: vec![
                    vec![0 as u64, 2 as u64, 4 as u64, 7 as u64],
                    vec![0 as u64, 1 as u64, 3 as u64],
                ],
            },
            TestCase {
                input_permutation: vec![3, 1, 2, 0],
                expected_summation_vector: vec![
                    vec![18 as u64, 20 as u64, 22 as u64, 22 as u64],
                    vec![6 as u64, 7 as u64, 9 as u64],
                ],
            },
            TestCase {
                input_permutation: vec![3, 2, 1, 0],
                expected_summation_vector: vec![
                    vec![18 as u64, 22 as u64, 23 as u64, 23 as u64],
                    vec![6 as u64, 8 as u64, 9 as u64]
                ],
            }
        ];

        for case in test_cases {
            let input_code =
                permutation_label::get_lehmer_code_from_permutation(&case.input_permutation);
            let size = (&case).input_permutation.len();
            let summation_vector =
                get_running_sum_of_item_pos_factorial_for_code(size, input_code);
            assert_eq!(summation_vector, case.expected_summation_vector);
        }
    }

    #[test]
    fn test_get_mask_sum_of_bits_position_factorial() {
        struct TestCase {
            size: usize,
            expected_output: Vec<u64>,
        }

        let test_cases = vec![
            TestCase{ size: 1, expected_output: vec![0, 1] },
            TestCase{ size: 2, expected_output: vec![0, 1, 1, 2] },
            TestCase{ size: 3, expected_output: vec![0, 1, 1, 2, 2, 3, 3, 4] }
        ];

        for case in test_cases {
            let output = get_mask_sum_of_bits_position_factorial(case.size);
            assert_eq!(output, case.expected_output);
        }
    }
}