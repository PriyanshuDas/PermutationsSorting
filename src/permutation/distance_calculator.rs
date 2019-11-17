use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;

const DEBUG_ENABLED: bool = true;
const INVALID_PAIR: (u8, u32) = (99, 99);
const  UNDEFINED: i64 = -2;

//todo: this is getting extremely complicated, streamline this process!
pub struct PermutationsData {
    //    visited[code][i][j] => perm_code 's block [i,j] has been visited before
    size: usize,
    visited: Vec<Vec<Vec<bool>>>,
    distance: Vec<u8>,
    //next_step: size, code
    next_step: Vec<(u8, u32)>,
    //[code][i][j] => j moved to after i in code => new code
    single_item_transposition_data: Vec<Vec<Vec<u32>>>,
    block_slide_reduction_memo: Vec<Vec<Vec<i64>>>,
    reduced_code: Vec<(usize, u32)>,
    pure_permutations: Vec<u32>,
}


//todo: space complexity can be reduced by closely packing some memos
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

    fn init_next_step(size: usize) -> Vec<(u8, u32)> {
        let mut next_step: Vec<(u8, u32)> = vec![];
        let lehmer_code_range = 0..constants::FACTORIALS[size];
        for code in lehmer_code_range {
            next_step.push(INVALID_PAIR);
        }
        next_step
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

    fn init_is_pure(size: usize) -> Vec<(usize, u32)> {
        let mut is_pure = vec![];

        for code in 0..constants::FACTORIALS[size] {
            let permutation = permutation_label::get_permutation_from_lehmer_code(
                size, code as usize);
            let reduced_permutation = permutation_helper::reduce_permutation(&permutation);
            if reduced_permutation.len() == permutation.len() {
                is_pure.push((size, code));
            } else {
                let reduced_code = permutation_label::get_lehmer_code_from_permutation(&reduced_permutation);
                is_pure.push((reduced_permutation.len(), reduced_code));
            }
        }
        return is_pure;
    }

    pub fn init_pure_permutations(&mut self) {
        for code in 0..self.reduced_code.len() {
            if self.reduced_code[code].0 == self.size {
                self.pure_permutations.push(code as u32);
            }
        }
    }

    pub fn init_block_slide_reduction_memo(size: usize) -> Vec<Vec<Vec<i64>>> {
        let mut block_slide_reduction_memo = vec![];
        let lehmer_code_range = constants::FACTORIALS[size];

        for code in 0..lehmer_code_range {
            let mut code_vector = vec![];
            for i in 0..size {
                let mut code_item_vector = vec![];
                //todo: could be optimized to use half memory
                for j in 0..size {
                    code_item_vector.push(UNDEFINED);
                }
                code_vector.push(code_item_vector);
            }
            block_slide_reduction_memo.push(code_vector);
        }
        block_slide_reduction_memo
    }

    pub fn init(size: usize) -> PermutationsData {
        let mut permutations_data = PermutationsData {
            size,
            next_step: PermutationsData::init_next_step(size),
            visited: PermutationsData::init_visited(size),
            distance: PermutationsData::init_distance(size),
            single_item_transposition_data: PermutationsData::init_single_item_transposition_data(size),
            pure_permutations: vec![],
            reduced_code: PermutationsData::init_is_pure(size),
            block_slide_reduction_memo: PermutationsData::init_block_slide_reduction_memo(size),
        };

        permutations_data.init_pure_permutations();
        return permutations_data;
    }

    //todo: write tests
    fn find_smallest_reducible_code_sliding_block_beyond(&mut self, cur_code: u32, i: usize, j: usize, k: usize) -> i64 {
        let mut new_code =
            adjacency_calculator::get_lehmer_code_by_moving_item_at_j_to_after_i(
                cur_code, k, i - 1);
        let block_memo_value = self.block_slide_reduction_memo[new_code][i][j];
        if block_memo_value != UNDEFINED {
            return block_memo_value;
        } else {
            let mut min_size_i_j = 99;
            let mut min_code_i_j = 99;
            let mut min_dist_i_j = 99;
            let mut cur_code = new_code;

            //todo: convert this into a tail recursion loop for updating values correctly
            // propagate the min_values from here upwards
            let future_min_code =
                if k+1 < self.size {
                    self.find_smallest_reducible_code_sliding_block_beyond(cur_code, i, j, k + 1)
                } else { (-1) };
            let cur_code_size = self.reduced_code[cur_code].0;
            let cur_code_code = self.reduced_code[cur_code].1;
            let cur_code_dist = self.distance[cur_code];


            if future_min_code == -1 || cur_code_dist < self.distance[future_min_code] {
                if self.reduced_code[cur_code].0 < self.size {
                    self.update_block_slide_reduction_memo(i, j, cur_code.clone(), cur_code.clone());
                    return cur_code_code as i64;
                }
                else {
                    self.block_slide_reduction_memo[new_code][i][j] = -1;
                    return -1;
                }
            } else if self.reduced_code[future_min_code].0 < self.size {
                return future_min_code;
            } else {
                return -1;
            }
        }
    }

    fn update_block_slide_reduction_memo(&mut self, i: usize, j: usize, cur_code: u32, new_code: u32) {
        if self.distance[cur_code][i][j] > self.distance[new_code] {
            self.block_slide_reduction_memo[cur_code][i][j] = new_code;
        }
    }

    //todo: min_size, min_code, min_dist could be packaged into single struct?
//todo: verify the inference of distance is not mistaken, i.e. no +-1 error when inheriting
    pub fn process_block_slide_reduction_memo(&mut self, permutations_data: &Vec<PermutationsData>) {
        for code in 0..constants::FACTORIALS[self.size] {
            let mut min_size = 99;
            let mut min_code = 99;
            let mut min_dist = 99;
            for i in 0..self.size {
                for j in i..self.size {
                    let min_code_in_block =
                        self.find_smallest_reducible_code_sliding_block_beyond(code, i, j, j+1);
                }
            }
            self.update_distance_and_next_step_for_code(code, min_dist, min_size, min_code);
        }
    }

    pub fn calculate(&self) {}

    //Complexity: O(n!*n^2)?
    pub fn update_init_on_basis_of_previous(&mut self, permutations_data: &Vec<PermutationsData>) {
        //Complexity: O(n!)
        self.update_distance_and_next_step_for_reducible_permutations(permutations_data);
        self.process_block_slide_reduction_memo(permutations_data);
        //Complexity: O(n!*n^2)
        self.update_distance_and_next_step_for_pure_permutations(permutations_data);
    }

    fn get_adjacent_reduced_permutation_to(&self, code: u32) -> Vec<u32> {
        let mut adjacent_codes = vec![];
        let position = vec![0 as usize; self.size];
        let permutation =
            permutation_label::get_permutation_from_lehmer_code(self.size, code as usize);

        for pos in 0..permutation.len() {
            position[item] = pos;
        }

        for i in 0..position.len() - 1 {
            for j in i..position.len() - 1 {
                let k1 =
                    if permutation[i] != 0 { position[permutation[i] - 1] } else { -1 };
                let k2 =
                    if permutation[j] != (self.size - 1) as u8 { position[permutation[j] + 1] } else { self.size };

                //todo: implement, complexity could be larger
                let code_1 = self.get_code_by_moving_block(code, i, j, k1);
                let code_2 = self.get_code_by_moving_block(code, i, j, k2);

                adjacent_codes.push(code_1);
                adjacent_codes.push(code_2);
            }
        }
        adjacent_codes
    }

    fn update_distance_and_next_step_for_pure_permutations(&mut self, permutations_data: &Vec<PermutationsData>) {
        for code in self.pure_permutations {
            //todo: implement
            let adjacent_reduced_permutations = self.get_adjacent_reduced_permutation_to(code);

            let mut min_dist = 99;
            let mut min_size = 99;
            let mut min_label = 99;

            for code in adjacent_reduced_permutations {
                let pair = self.reduced_code[code];
                let new_size = pair.0;
                let new_label = pair.1;
                let new_dist = permutations_data[new_size].distance[new_label] + 1;

                if new_dist < min_dist {
                    min_dist = new_dist;
                    min_size = new_size;
                    min_label = new_label;
                }
            }
            self.update_distance_and_next_step_for_code(code, min_dist, min_size, min_label);
        }
    }

    fn update_distance_and_next_step_for_code(&mut self, code: u32, new_dist: u8, new_size: u8, new_code: u32) {
        self.distance[code] = new_dist;
        self.next_step[code] = (new_size, new_code);
    }

    fn update_distance_and_next_step_for_reducible_permutations(&mut self, permutations_data: &Vec<PermutationsData>) {
        let x = for code in 0..self.reduced_code.len() {
            let size_code_pair = self.reduced_code[code];
            if size == self.size {
                continue;
            } else {
                let new_size = size_code_pair.0;
                let new_code = size_code_pair.1;
                let new_dist = permutations_data[new_size].distance[new_code];
                self.update_distance_and_next_step_for_code(code as u32, new_dist, new_size as u8, new_code);
            }
        };
    }

    pub fn process_pure_permutations(&mut self) {}
}


pub fn generate_data_for_size_up_to(max_size: usize) {
    let mut permutations_data = vec![
        PermutationsData {
            size: 0,
            visited: vec![],
            distance: vec![],
            next_step: vec![],
            single_item_transposition_data: vec![],
            //todo: fill?
            block_slide_reduction_memo: vec![],
            reduced_code: vec![],
            pure_permutations: vec![],
        },
        PermutationsData {
            size: 1,
            visited: vec![vec![vec![true]]],
            distance: vec![0],
//todo: verify?
            next_step: vec![(0, 0)],
            single_item_transposition_data: vec![],
            //todo: fill?
            block_slide_reduction_memo: vec![],
            reduced_code: vec![(0, 0)],
            pure_permutations: vec![],
        }
    ];
    for size in 2..max_size {
        let mut cur_size_permutations_data = PermutationsData::init(size);
        cur_size_permutations_data.update_init_on_basis_of_previous(&permutations_data);
        cur_size_permutations_data.process_pure_permutations();
        permutations_data.push(cur_size_permutations_data);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        size: u8,
        permutations_data: PermutationsData,
        expected_pure_permutations: Vec<Vec<u8>>,
    }

    //todo: add test for init of next step
    #[test]
    fn test_init_permutations_data() {
        if DEBUG_ENABLED {
            println!("\t[test_init_permutations_data]");
        }

        let test_cases = vec![
            TestCase {
                size: 3,
                permutations_data: PermutationsData::init(3),
                expected_pure_permutations: vec![vec![2, 1, 0]],
            },
            TestCase {
                size: 4,
                permutations_data: PermutationsData::init(4),
                expected_pure_permutations: vec![
                    vec![1, 0, 3, 2],
                    vec![1, 3, 0, 2],
                    vec![1, 3, 2, 0],
                    vec![2, 0, 3, 1],
                    vec![2, 1, 3, 0],
                    vec![3, 0, 2, 1],
                    vec![3, 1, 0, 2],
                    vec![3, 2, 1, 0]
                ],
            },
        ];

        for case in test_cases {
            if DEBUG_ENABLED {
                println!("==========[Test Case]==========");
            }
            assert_pure_permutations(&case);
            assert_reduced_code(&case);
        }
    }

    fn assert_reduced_code(case: &TestCase) {
        let pure_labels = case.permutations_data.pure_permutations.clone();
        for pure_label in pure_labels {
            let is_pure =
                case.permutations_data.reduced_code[pure_label as usize].0
                    == case.size as usize;

            assert_eq!(is_pure, true);
        }
    }

    fn assert_pure_permutations(case: &TestCase) {
        let mut pure_permutations = vec![];
        let pure_labels = case.permutations_data.pure_permutations.clone();
        for label in pure_labels {
            let permutation =
                permutation_label::get_permutation_from_lehmer_code(case.size as usize, label as usize);
            pure_permutations.push(permutation);
        }
        assert_eq!(pure_permutations, case.expected_pure_permutations);
    }
}