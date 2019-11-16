use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;

struct PermutationsData {
    //    visited[code][i][j] => perm_code 's block [i,j] has been visited before
    visited: Vec<Vec<Vec<bool>>>,
    distance: Vec<u8>,
    //[code][i][j] => j moved to after i in code => new code
    single_item_transposition_data: Vec<Vec<Vec<u32>>>,
    pure_permutations: Vec<u32>,
}

impl PermutationsData {
    fn init_visited(size: usize) -> Vec<Vec<Vec<bool>>> {
        let mut visited: Vec<Vec<Vec<bool>>> = vec![];
        let lehmer_code_range = 0..constants::FACTORIALS[size];
        for code in lehmer_code_range {
            let mut current_code: Vec<Vec<bool>> = vec![];
            for i in 1..size {
                let mut current_start_pos: Vec<bool> = vec![];
                for j in 0..i - 1 {
                    current_start_pos.push(false);
                }
                current_code.push(current_start_pos);
            }
            visited.push(current_code)
        }
        visited
    }

    fn init_distance(size: usize) -> Vec<u8> {
        let mut distance: Vec<u8> = vec![];
        let lehmer_code_range = constants::FACTORIALS[size];
        for i in 0..lehmer_code_range {
            distance.push(99);
        }
        distance
    }

    fn init_single_item_transposition_data(size: usize) -> Vec<Vec<Vec<u32>>> {
        let mut single_item_transposition_memo = vec![];
        let lehmer_code_range = constants::FACTORIALS[size];

        for code in 0..lehmer_code_range {
            let mut code_vector = vec![];
            for j in 0..size {
                let mut code_item_vector = vec![];
                for i in 0..j {
                    code_item_vector.push(
                        adjacency_calculator::get_lehmer_code_by_moving_item_at_j_to_after_i(
                            code, i, j))
                }
                code_vector.push(code_item_vector);
            }
            single_item_transposition_memo.push(code_vector);
        }
        single_item_transposition_memo
    }

    pub fn init(size: usize) -> PermutationsData {
        PermutationsData {
            visited: PermutationsData::init_visited(size),
            distance: PermutationsData::init_distance(size),
            single_item_transposition_data: PermutationsData::init_single_item_transposition_data(size),
            pure_permutations: permutation_label::get_all_pure_lehmer_codes_of_size(size as u8),
        }
    }

    pub fn calculate(&self) {}
}


#[cfg(test)]
mod tests {
    use super::*;
    # [test]
    fn test_init_permutations_data() {
        let permutations_data = PermutationsData::init(3);
        println!("AAAA");
        println!("{:?}", permutation_label::get_permutation_from_lehmer_code(3, permutations_data.pure_permutations[0] as usize));
        assert_eq!(permutations_data.pure_permutations.len(), 1);
        assert_eq!(1, 0);
    }
}