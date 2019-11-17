use crate::permutation::permutation_label::get_permutation_from_label;
use crate::permutation::constants;
use super::permutation_label;

use crate::permutation::permutations_generator;

pub struct AdjacencyCalculator {
    //array:
    // starting point, size, max bits
    //name: lehmer_delta
    lehmer_delta: Vec<Vec<Vec<u32>>>,
    lehmer_delta_min: Vec<u32>,
    //[l][r] gives value with all bits in [l, r] set to 1
    range_bitmasks: Vec<Vec<u16>>,
}

//[start_pos][block_size][mask]
impl AdjacencyCalculator {
    pub fn init(permutation_size: u8) -> AdjacencyCalculator {

        //todo: verify lehmer_delta_min_memo calculation
        let mut lehmer_delta_min_memo: Vec<u32> = vec![0; (1 << permutation_size) as usize];
        for mask in 0..(1 << permutation_size) {
            let mut current_sum = 0;
            for bit in 0..permutation_size {
                if 1 << bit & mask > 0 {
                    current_sum += constants::get_factorial((permutation_size - bit) as usize)
                }
            }
            lehmer_delta_min_memo[mask] = current_sum;
        }

        //todo: write proper tests for lehmer_delta
        //todo: lemer_delta probably not needed
        let mut lehmer_delta_memo: Vec<Vec<Vec<u32>>> = vec![];
        let mut range_bitmasks_memo: Vec<Vec<u16>> = vec![];
        for start_pos in 0..permutation_size {
            let mut current_start_pos_row: Vec<Vec<u32>> = vec![];
            for block_size in 0..=(permutation_size - start_pos) {
                let mut current_block_size_row = vec![];
                for mask in 0..(1 << (block_size + 1)) {
                    let mut current_sum = 0;
                    for bit in start_pos..start_pos + block_size {
                        if 1 << bit & mask > 0 {
                            current_sum += constants::get_factorial(bit as usize)
                        }
                    }
                    current_block_size_row.push(current_sum);
                }
                current_start_pos_row.push(current_block_size_row.clone());
            }
            lehmer_delta_memo.push(current_start_pos_row);
        }

        for i in 0..16 {
            let mut bit_value: u16 = 1 << i;
            let mut mask_row = vec![];
            for j in i..16 {
                bit_value |= 1 << j;
                mask_row.push(bit_value);
            }
            range_bitmasks_memo.push(mask_row);
        }

        let adjacency_calculator = AdjacencyCalculator {
            lehmer_delta: lehmer_delta_memo,
            lehmer_delta_min: lehmer_delta_min_memo,
            range_bitmasks: range_bitmasks_memo,
        };
        return adjacency_calculator;
    }
}

const DEBUG_ENABLED: bool = true;

//todo: implement cleanly
//todo: case when new_after_pos is -1
pub fn get_code_shifting_item_to_new_position(
    lehmer_code: u32,
    original_pos: usize,
    new_after_pos: i8,
) -> u32 {
    return 0;
}

//todo: extract into a config? too many params here
// has become a monolith, divide and modularize
// otherwise will become hard to debug and test
// consider:
// make a config
// make permutation_precompute a struct and expose methods to cleanly extract req. data
// make inversion_bitmap a struct and expose methods

//todo: fix bugs
//todo: make O(1) consistently?
fn get_lehmer_code_by_moving_block_by_delta(
    adjacency_calculator: &AdjacencyCalculator,
    original_permutation: &Vec<u8>,
    curr_code: u32,
    block_start: usize,
    block_size: usize,
    delta: usize,
    permutation_precompute: &Vec<Vec<u32>>,
    inversion_bitmap: &Vec<u16>) -> u32
{
    let block_start_pos_orig = block_start;
    let block_end_pos_orig = block_start + block_size - 1;

    let block_start_pos_cur = block_start_pos_orig + delta - 1;
    let block_end_pos_cur = block_end_pos_orig + delta - 1;

    let block_start_pos_new = block_end_pos_orig + delta;
    let block_end_pos_new = block_end_pos_orig + delta;

    let item_moved_original_pos = block_start + block_size + delta - 1;
    let item_moved_new_pos = block_start + delta - 1;

    let item_to_be_moved_value = original_permutation[item_moved_original_pos];
    let n = original_permutation.len();

    if DEBUG_ENABLED {
        println!("==========[get_lehmer_code_by_moving_block_by_delta called]==========");
        println!("\t block_start_pos_original = {}\n\
        \t block_end_pos_original = {}\n\
        \t item_moved_original_pos = {}\n\
        \t item_moved_new_pos = {}\n\
        \t item_to_be_moved_value = {}\n",
                 block_start_pos_orig,
                 block_end_pos_orig,
                 item_moved_original_pos,
                 item_moved_new_pos,
                 item_to_be_moved_value);
    }

    //item's current_contribution to code:
    // (item_to_be_moved - sum_bits(inversion_bitmap[item_to_be_moved]_0_item_original_pos)))
    // *item_original_pos!
    let inv_mask_for_items_gt_item_to_left_at_old_pos =
        (inversion_bitmap[item_to_be_moved_value as usize]
            & adjacency_calculator.range_bitmasks[0][item_moved_original_pos - 1]) as usize;

    let inv_mask_for_items_greater_than_item_in_block =
        (inversion_bitmap[item_to_be_moved_value as usize]
            & adjacency_calculator.range_bitmasks[block_start_pos_orig][block_end_pos_orig]) as usize;

    //todo: fix next
    let ct_items_gt_item_to_left_orig_pos =
        inv_mask_for_items_gt_item_to_left_at_old_pos.count_ones();
    let ct_items_gt_item_in_block =
        block_size as u32 - inv_mask_for_items_greater_than_item_in_block.count_ones();

    let ct_items_lt_item_to_left_original_pos: u8 =
        (item_moved_original_pos as u32
            - ct_items_gt_item_to_left_orig_pos) as u8;
    let items_lesser_than_item_to_left_new_pos: u8 =
        ct_items_lt_item_to_left_original_pos
            - (ct_items_gt_item_in_block) as u8;

    let value_for_item_at_old_pos = (item_to_be_moved_value -
        ct_items_lt_item_to_left_original_pos) as i64
        * constants::FACTORIALS[n - item_moved_original_pos - 1 as usize] as i64;

    let value_for_item_at_new_pos = (item_to_be_moved_value
        - items_lesser_than_item_to_left_new_pos as u8) as i64
        * constants::FACTORIALS[n - item_moved_new_pos - 1 as usize] as i64;

    let delta_value_for_item_move: i64 =
        value_for_item_at_new_pos as i64 - value_for_item_at_old_pos as i64;

    if DEBUG_ENABLED {
        println!("\titems_lesser_to_item_to_left_original_pos: {}\n\
        \titems_lesser_than_item_to_left_new_pos: {}\n\
        \tvalue_for_item_at_new_pos: {}\n\
        \tvalue_for_item_at_old_pos: {}",
                 ct_items_lt_item_to_left_original_pos,
                 items_lesser_than_item_to_left_new_pos,
                 value_for_item_at_new_pos,
                 value_for_item_at_old_pos);
    }

    let old_block_value_for_position =
        get_block_value_for_delta(delta - 1,
                                  permutation_precompute,
                                  block_start_pos_cur,
                                  block_end_pos_cur);

    let new_block_value_for_position =
        get_block_value_for_delta(delta,
                                  permutation_precompute,
                                  block_start_pos_new,
                                  block_end_pos_new);

    if DEBUG_ENABLED {
        println!("Calculating Block Value: \
        \n\told_block_value_for_position: {}\
        \n\tnew_block_value_for_position: {}\
        \n\tinv_mask_for_items_less_than_item_in_block: {}",
                 old_block_value_for_position,
                 new_block_value_for_position,
                 inv_mask_for_items_greater_than_item_in_block);
    }
    let delta_value_for_block_shift: i64 = new_block_value_for_position as i64
        - old_block_value_for_position as i64;

    //todo: fix this!

    let delta_value_on_block_due_to_item_shift =
        adjacency_calculator
            .lehmer_delta_min[inv_mask_for_items_greater_than_item_in_block as usize];

    if DEBUG_ENABLED {
        println!("\tdelta_value_for_item_move: {}\
        \n\tdelta_value_for_block_shift: {}\
        \n\tdelta_value_on_block_due_to_item_shift: {}",
                 delta_value_for_item_move,
                 delta_value_for_block_shift,
                 delta_value_on_block_due_to_item_shift);
    }

    let lehmer_delta = (delta_value_for_item_move +
        delta_value_for_block_shift) as i64;

    println!("curr_code: {}, lehmer_delta: {}", curr_code, lehmer_delta);
    return (curr_code as i64 + lehmer_delta) as u32;
}

fn get_block_value_for_delta(delta: usize,
                             permutation_precompute: &Vec<Vec<u32>>,
                             block_start_pos: usize,
                             block_end_pos: usize) -> u32 {
    if DEBUG_ENABLED {
        println!("\n===========[get_block_value_for_delta called]======================\
        \n\tdelta: {},\
        \n\tblock_start_pos: {},\
         \n\tblock_end_pos: {})",
                 delta, block_start_pos, block_end_pos);
    }
    let sum_sequence_before_block_start =
        if block_start_pos > 0 {
            permutation_precompute[delta][block_start_pos - 1]
        } else { 0 };
    let sum_sequence_at_block_end_pos = permutation_precompute[delta][block_end_pos];
    let block_value =
        sum_sequence_at_block_end_pos - sum_sequence_before_block_start;

    if DEBUG_ENABLED {
        println!("\n\tsum_sequence_before_block_start: {}\
        \n\tsum_sequence_at_block_end_pos: {}",
                 sum_sequence_before_block_start,
                 sum_sequence_at_block_end_pos)
    }
    return block_value;
}

//todo: implement, might need to inc to u32 for larger permutations
fn get_inversion_bitmap_for_permutation(permutation: &Vec<u8>) -> Vec<u16> {
    let mut inversion_bitmap: Vec<u16> = vec![0; permutation.len()];
    let size = permutation.len();
    for item in permutation {
        let mut mask: u16 = 0;
        for pos in 0..size {
            if permutation[pos] > *item {
                mask |= (1 << ((size - 1 - pos) as u16));
            }
        }
        inversion_bitmap[(*item) as usize] = mask;
    }
    return inversion_bitmap;
}



pub fn get_adjacent_labels_for_lehmer_number(size: usize, lehmer_number: u32) {
    let adjacency_calculator = AdjacencyCalculator::init(size as u8);
    let permutation: Vec<u8> = get_permutation_from_label(lehmer_number);
    let permutation_precompute = get_precompute_for_permutation(&permutation);
    let inversion_bitmap = get_inversion_bitmap_for_permutation(&permutation);

    for i in 0..size {
        for j in i..size {
            let mut current_lehmer_code = lehmer_number;
            for k in j + 1..size {
                let new_lehmer_code =
                    get_lehmer_code_by_moving_block_by_delta(
                        &adjacency_calculator,
                        &permutation,
                        current_lehmer_code,
                        i,
                        j - i + 1,
                        k - j,
                        &permutation_precompute,
                        &inversion_bitmap,
                    );
                current_lehmer_code = new_lehmer_code;
            }
        }
    }
}

fn get_precompute_for_permutation(perm: &Vec<u8>) -> Vec<Vec<u32>> {
    let n = perm.len();
    let mut memo = init_2d_vector(n);
    for delta in 0..n {
        let mut sum: u32 = 0;
        for pos in delta..n {
            sum += (perm[pos - delta as usize] as u32)
                * constants::FACTORIALS[(n - (pos) - 1) as usize];
            memo[delta][pos] = sum;
        }
    }
    memo
}

fn init_2d_vector(n: usize) -> Vec<Vec<u32>> {
    let mut memo = vec![];
    for i in 0..n {
        let mut vec_i = vec![];
        for j in 0..n {
            vec_i.push(0);
        }
        memo.push(vec_i);
    }
    memo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permutation::permutation_label::{get_permutation_from_lehmer_code, get_lehmer_code_from_permutation};

    #[test]
    pub fn test_bit_operations() {
        assert_eq!(1 << 4, 16);
        assert_ne!(1 << 4, 15);
    }


    //todo: make some asserts here
    #[test]
    pub fn test_init() {
        let adjacency_calculator = AdjacencyCalculator::init(3);
        for start_pos_row in adjacency_calculator.lehmer_delta {
            for block_size_column in start_pos_row {
                print!("[");
                for position in 0..block_size_column.len() {
                    print!("({:b}, {}) ", position, block_size_column[position]);
                }
                print!("]");
            }
            println!();
        }

        for start_pos in adjacency_calculator.range_bitmasks {
            print!("[ ");
            for end_pos_mask in start_pos {
                print!("{:16b} ", end_pos_mask);
            }
            println!("] ");
        }
    }

    #[test]
    pub fn test_init_3d_vector() {
        let expected_output: Vec<Vec<u32>> = vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 0, 0]];
        let actual_output = init_2d_vector(3);
//    println!("{:?}", actual_output);
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn test_precompute_for_permutation() {
        let input_permutations = vec![
            vec![0, 1, 2, 3],
            vec![3, 2, 1, 0],
            vec![2, 3, 1, 0]
        ];
        let expected_output: Vec<Vec<Vec<u32>>> = vec![
            vec![
                vec![0, 2, 4, 7],
                vec![0, 0, 1, 3],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 0]
            ],
            vec![
                vec![18, 22, 23, 23],
                vec![0, 6, 8, 9],
                vec![0, 0, 3, 5],
                vec![0, 0, 0, 3]
            ],
            vec![
                vec![12, 18, 19, 19],
                vec![0, 4, 7, 8],
                vec![0, 0, 2, 5],
                vec![0, 0, 0, 2]
            ],
        ];

        for i in 0..input_permutations.len() {
            let input_permutation = input_permutations[i as usize].clone();
            let output_precompute =
                get_precompute_for_permutation(&input_permutation);
            assert_eq!(output_precompute, expected_output[i]);
            if DEBUG_ENABLED {
                permutations_generator::print_2d_vector(output_precompute);
            }
            println!();
        }
    }

    //todo: finish test
    #[test]
    fn test_get_inversion_bitmap_for_permutation() {
        let input_permutations: Vec<Vec<u8>> = vec![
            vec![0, 1, 2, 3],
            vec![3, 2, 1, 0],
            vec![2, 3, 1, 0]
        ];
        let expected_output: Vec<Vec<u16>> = vec![
            vec![0b0111, 0b0011, 0b0001, 0b0000],
            vec![0b1110, 0b1100, 0b1000, 0b0000],
            vec![0b1110, 0b1100, 0b0100, 0b0000],
        ];

        for pos in 0..input_permutations.len() {
            let output =
                get_inversion_bitmap_for_permutation(&input_permutations[pos]);
            if DEBUG_ENABLED {
                println!("{:?}", output);
            }
            assert_eq!(output, expected_output[pos]);
        }
    }

    #[test]
    fn test_get_lehmer_code_after_moving_jth_item_to_after_i() {
        struct TestCase {
            permutation: Vec<u8>,
            block_start: usize,
            block_size: usize,
            delta_shift: usize,
            expected_output: Vec<u8>,
        }

        impl TestCase {
            pub fn print_case(&self) {
                println!(" [Test Case: ]\n\
                \tpermutation: {:?}, \n\
                \tblock_start: {}, \n\
                \tblock_size: {}, \n\
                \tdelta_shift: {}, \n\
                \texpected_output: {:?}",
                         self.permutation, self.block_start,
                         self.block_size, self.delta_shift,
                         self.expected_output)
            }
        }

        let test_cases = vec![
            TestCase {
                permutation: vec![0, 1, 2, 3],
                block_start: 0,
                block_size: 3,
                delta_shift: 1,
                expected_output: vec![3, 0, 1, 2],
            },
            TestCase {
                permutation: vec![0, 1, 2, 3],
                block_start: 0,
                block_size: 2,
                delta_shift: 1,
                expected_output: vec![2, 0, 1, 3],
            },
            TestCase {
                permutation: vec![0, 1, 2, 3],
                block_start: 1,
                block_size: 2,
                delta_shift: 1,
                expected_output: vec![0, 3, 1, 2],
            },
            TestCase {
                permutation: vec![3, 2, 1, 0],
                block_start: 0,
                block_size: 3,
                delta_shift: 1,
                expected_output: vec![0, 3, 2, 1],
            },
            TestCase {
                permutation: vec![2, 3, 1, 0],
                block_start: 0,
                block_size: 1,
                delta_shift: 3,
                expected_output: vec![3, 1, 0, 2],
            },
        ];
        let adjacency_calculator = AdjacencyCalculator::init(4);

        for test_case in test_cases {
            let permutation_precompute =
                get_precompute_for_permutation(&test_case.permutation);
            let inversion_bitmap = get_inversion_bitmap_for_permutation(
                &test_case.permutation);
            if DEBUG_ENABLED {
                test_case.print_case();
            }
            let lehmer_code = get_lehmer_code_from_permutation(&test_case.permutation);
            let new_lehmer_code =
                get_lehmer_code_by_moving_block_by_delta(
                    &adjacency_calculator,
                    &test_case.permutation,
                    lehmer_code,
                    test_case.block_start, test_case.block_size, test_case.delta_shift,
                    &permutation_precompute, &inversion_bitmap);

            let new_permutation =
                get_permutation_from_lehmer_code(
                    test_case.permutation.len(),
                    new_lehmer_code as usize);

            let expected_lehmer_code =
                get_lehmer_code_from_permutation(&test_case.expected_output);

            let expected_lehmer_delta = expected_lehmer_code as i64 - lehmer_code as i64;
            let actual_lehmer_delta = new_lehmer_code as i64 - lehmer_code as i64;

            if DEBUG_ENABLED {
                println!("======[Test Case Analysis]======\
                \n\tOriginal Permutation: {:?}\
                \n\tNew Actual Permutation: {:?}\
                \n\tNew Expected Permutation: {:?}\
                \n\tNew Actual Lehmer Code: {}\
                \n\tNew Expected Lehmer Code: {}\
                \n\tOld Lehmer Code: {}\
                \n\tExpected Lehmer Delta: {}\
                \n\tActual Lehmer Delta: {}\
                \n=============================",
                         test_case.permutation,
                         new_permutation,
                         test_case.expected_output,
                         new_lehmer_code,
                         expected_lehmer_code,
                         lehmer_code,
                         expected_lehmer_delta,
                         actual_lehmer_delta);
            }
        }
    }
}