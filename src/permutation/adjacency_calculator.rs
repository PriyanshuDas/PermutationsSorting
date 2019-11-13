use crate::permutation::permutation_label::get_permutation_from_label;
use crate::permutation::constants;

struct AdjacencyCalculator {
    //array:
    // starting point, size, max bits
    //name: lehman_delta
    lehman_delta: Vec<Vec<Vec<u32>>>
}

impl AdjacencyCalculator {
    //todo: write proper tests for this
    pub fn init(permutation_size: u8) -> AdjacencyCalculator {
        let mut lehman_delta_memo: Vec<Vec<Vec<u32>>> = vec![];
        //todo: finish this

        for start_pos in 0..permutation_size {
            let mut current_start_pos_row: Vec<Vec<u32>> = vec![];
            for block_size in 0..=(permutation_size-start_pos) {
                let mut current_block_size_row = vec![];
                for mask in 0..(1<<(block_size+1)) {
                    let mut current_sum = 0;
                    for bit in start_pos..start_pos+block_size {
                        if 1<<bit & mask > 0 {
                            current_sum += constants::get_factorial(bit as usize)
                        }
                    }
                    current_block_size_row.push(current_sum);
                }
                current_start_pos_row.push(current_block_size_row.clone());
            }
            lehman_delta_memo.push(current_start_pos_row);
        }

        let adjacency_calculator = AdjacencyCalculator{lehman_delta: lehman_delta_memo};
        return adjacency_calculator;
    }
}


pub fn get_lehman_code_after_moving_jth_item_to_after_i(
    curr_code: u32, old_pos: usize, after_pos: usize) -> u32 {
    //todo: finish this
    return 0;
}

pub fn get_adjacent_labels_for_lehman_number(size: usize, lehman_number: u32) {
    let permutation: Vec<u8> = get_permutation_from_label(lehman_number);
    for i in 0..size {
        for j in i..size {
            let mut current_lehman_code = lehman_number;
            for k in j + 1..size {
                let new_lehman_code =
                    get_lehman_code_after_moving_jth_item_to_after_i(current_lehman_code, k, i - 1);
                current_lehman_code = new_lehman_code;
            }
        }
    }
}
#[test]
pub fn test_bit_operations() {
    assert_eq!( 1 << 4, 16);
    assert_ne!(1 << 4, 15);
}

#[test]
pub fn test_init() {
    let adjacency_calculator = AdjacencyCalculator::init(3);
    for start_pos_row in adjacency_calculator.lehman_delta {
        for block_size_column in start_pos_row {
            print!("[");
            for position in 0..block_size_column.len() {
                print!("({:b}, {}) ", position, block_size_column[position]);
            }
            print!("]");
        }
        println!();
    }
}

//todo: test and verify lehman's code generation algorithm