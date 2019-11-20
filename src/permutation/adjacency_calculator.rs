use crate::permutation::permutation_label::get_permutation_from_label;
use crate::permutation::constants;
use super::permutation_label;

use crate::permutation::permutations_generator;
use std::cmp::max;
use rayon::prelude::*;

const DEBUG_ENABLED: bool = false;

struct AdjacencyCalculator {
    size: usize,
    permutations: Vec<Vec<u8>>,
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
    return (0..constants::FACTORIALS[size] as usize)
        .collect::<Vec<usize>>().par_iter_mut()
        .map(|code| get_mask_for_pos_with_items_lesser_than_item_for_code(size, *code as u32))
        .collect();
}

fn get_running_sum_of_item_pos_factorial_for_all_codes(size: usize) -> Vec<Vec<Vec<u64>>> {
    return (0..constants::FACTORIALS[size] as usize)
        .collect::<Vec<usize>>().par_iter_mut()
        .map(|code| get_running_sum_of_item_pos_factorial_for_code(size, *code as u32))
        .collect();
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
    return (0..(1 << size) as u32)
        .collect::<Vec<u32>>()
        .par_iter_mut()
        .map(|mask| get_position_factorial_for_set_bits_in_mask(size, *mask))
        .collect();
}

fn get_permutations_of_size(size: usize) -> Vec<Vec<u8>> {
    return (0..constants::FACTORIALS[size] as usize).collect::<Vec<usize>>()
        .par_iter_mut()
        .map(|code|
            permutation_label::get_permutation_from_lehmer_code(size, *code as usize))
        .collect();
}

fn get_mask_with_bits_set_before(old_pos: i32) -> i32 {
    if old_pos <= 0 {
        return 0;
    }
    (1 << (old_pos as i32)) - 1
}

impl AdjacencyCalculator {
    pub fn init(size: usize) -> AdjacencyCalculator {
        let permutations =
            get_permutations_of_size(size);
        let mask_for_pos_with_items_greater_than_item_for_permutation =
            get_masks_for_pos_with_items_lesser_than_item_for_permutation(size);
        let running_sum_of_item_pos_factorial =
            get_running_sum_of_item_pos_factorial_for_all_codes(size);
        let mask_sum_of_bits_position_factorial =
            get_mask_sum_of_bits_position_factorial(size);
        return AdjacencyCalculator {
            size,
            permutations,
            mask_for_pos_with_items_greater_than_item_for_permutation,
            running_sum_of_item_pos_factorial,
            mask_sum_of_bits_position_factorial,
        };
    }

    //todo: can be modularized?
    fn get_delta_for_moving_item(&self, code: u32, old_pos: usize, new_pos: i8) -> i64 {
        let size = self.size;
        let mut delta: i64 = 0;
        let block_running_sum_delta = self.get_block_running_sum_delta(code, old_pos, new_pos);
        let item_move_delta = self.get_item_move_delta(code, old_pos, new_pos);
        let block_delta_due_to_item_move = self.get_block_delta_due_to_item_move(code, old_pos, new_pos);
        block_running_sum_delta + item_move_delta + block_delta_due_to_item_move
    }

    //todo: complete and test
    fn get_block_delta_due_to_item_move(&self, code: u32, old_pos: usize, new_pos: i8) -> i64 {
        let item = self.permutations[code as usize][old_pos];
        let lower_than_item_mask =
            self.mask_for_pos_with_items_greater_than_item_for_permutation[code as usize][item as usize];

        let block_mask =
            get_mask_with_bits_set_before(old_pos as i32)
                ^ get_mask_with_bits_set_before((new_pos + 1) as i32);

        self.mask_sum_of_bits_position_factorial[block_mask as usize] as i64
    }

    fn get_item_move_delta(&self, code: u32, old_pos: usize, new_pos: i8) -> i64 {
        let item_to_be_moved = self.permutations[code as usize][old_pos];
        let lower_than_item_mask =
            self.mask_for_pos_with_items_greater_than_item_for_permutation[code as usize][item_to_be_moved as usize];
        let original_position_range_to_left_mask: i32 = get_mask_with_bits_set_before(old_pos as i32);
        let new_position_to_left_mask: i32 = get_mask_with_bits_set_before(new_pos as i32);
        let mask_for_sum_of_lower_items_to_left_original_pos =
            (original_position_range_to_left_mask & lower_than_item_mask as i32);
        let mask_for_sum_of_lower_items_to_left_new_pos =
            (new_position_to_left_mask & lower_than_item_mask as i32);

        self.mask_sum_of_bits_position_factorial[mask_for_sum_of_lower_items_to_left_new_pos as usize] as i64
            - self.mask_sum_of_bits_position_factorial[mask_for_sum_of_lower_items_to_left_original_pos as usize] as i64
    }

    //todo: overflow fixes
    fn get_block_running_sum_delta(&self, code: u32, old_pos: usize, new_pos: i8) -> i64 {
        let block_original_right = self.running_sum_of_item_pos_factorial[code as usize][old_pos - 1][0];
        let block_original_left =
            if new_pos >= 0 { self.running_sum_of_item_pos_factorial[code as usize][new_pos as usize][0] } else { 0 };

        let block_original_running_sum = block_original_right - block_original_left;

        let block_new_right = self.running_sum_of_item_pos_factorial[code as usize][old_pos - 1][1];
        let block_new_left =
            if new_pos >= 0 { self.running_sum_of_item_pos_factorial[code as usize][new_pos as usize][1] } else { 0 };

        let block_new_running_sum = block_new_right - block_new_left;

        block_new_running_sum as i64 - block_original_running_sum as i64
    }
    //todo: implement
    pub fn get_single_item_transposition_memo(&self) -> Vec<Vec<Vec<u32>>> {
        let memo = vec![];
        for code in 0..constants::FACTORIALS[self.size] {
            for i in -1..(self.size as i8 - 2) {
                for j in i + 2..self.size as i8 {
                    let code_delta =
                        self.get_delta_for_moving_item(code, j as usize, i);
                }
            }
        }
        memo
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
            TestCase { size: 1, expected_output: vec![0, 1] },
            TestCase { size: 2, expected_output: vec![0, 1, 1, 2] },
            TestCase { size: 3, expected_output: vec![0, 1, 1, 2, 2, 3, 3, 4] }
        ];

        for case in test_cases {
            let output = get_mask_sum_of_bits_position_factorial(case.size);
            assert_eq!(output, case.expected_output);
        }
    }

    #[test]
    fn test_adjacency_calculator_init() {
        let adjacency_calculator = AdjacencyCalculator::init(12);
    }

    #[test]
    fn test_get_masks_for_pos_with_items_lesser_than_item_for_permutation() {
        let output =
            get_masks_for_pos_with_items_lesser_than_item_for_permutation(10);
    }

    #[test]
    fn test_get_masks_for_pos_with_items_lesser_than_item_for_permutation_for_small() {
        let output =
            get_masks_for_pos_with_items_lesser_than_item_for_permutation(3);
        let expected_output = vec![
            vec![0, 1, 3],
            vec![0, 1, 5],
            vec![0, 2, 3],
            vec![0, 4, 5],
            vec![0, 2, 6],
            vec![0, 4, 6]];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_get_permutations_of_size_for_small() {
        let output = get_permutations_of_size(3);
        let expected_output = vec![
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0]];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_time_get_permutations_of_size_for_large() {
        let output = get_permutations_of_size(10);
    }

    #[test]
    fn test_get_running_sum_of_item_pos_factorial_for_all_codes_for_small() {
        let output =
            get_running_sum_of_item_pos_factorial_for_all_codes(3);
        let expected_output = vec![
            vec![
                vec![0, 1, 3],
                vec![0, 1]],
            vec![
                vec![0, 2, 3],
                vec![0, 2]],
            vec![
                vec![2, 2, 4],
                vec![1, 1]],
            vec![
                vec![2, 4, 4],
                vec![1, 3]],
            vec![
                vec![4, 4, 5],
                vec![2, 2]],
            vec![
                vec![4, 5, 5],
                vec![2, 3]]];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_time_get_running_sum_of_item_pos_factorial_for_all_codes_for_large() {
        let output =
            get_running_sum_of_item_pos_factorial_for_all_codes(10);
    }

    #[test]
    fn test_get_mask_sum_of_bits_position_factorial_for_small() {
        let output =
            get_running_sum_of_item_pos_factorial_for_all_codes(3);
        let expected_output = vec![
            vec![
                vec![0, 1, 3],
                vec![0, 1]],
            vec![
                vec![0, 2, 3],
                vec![0, 2]],
            vec![
                vec![2, 2, 4],
                vec![1, 1]],
            vec![
                vec![2, 4, 4],
                vec![1, 3]],
            vec![
                vec![4, 4, 5],
                vec![2, 2]],
            vec![
                vec![4, 5, 5],
                vec![2, 3]]];
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_time_get_mask_sum_of_bits_position_factorial_for_all_codes_for_large() {
        let output =
            get_mask_sum_of_bits_position_factorial(13);
    }
}