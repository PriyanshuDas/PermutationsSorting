use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;
use std::cmp::{min, max};
use crate::permutation::adjacency_calculator::get_code_shifting_item_to_new_position;

const DEBUG_ENABLED: bool = false;
const INVALID_PAIR: (u8, u32) = (99, 99);
const UNDEFINED: i64 = -2;

//todo: this is getting extremely complicated, streamline this process!

//not working
// next_step
// distance
//
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
    fn print_single_item_transposition_data(&self) {}
    fn print_block_slide_reduction_memo(&self) {}
    fn print_visited(&self) {}
    pub fn print(&self) {
        println!("================================[Data for size = {}]================================", self.size);
        println!("\tsize: {:?}", self.size);
        println!("\tpure_permutations:\n\t{:?}", self.pure_permutations);
        println!("\tnext_step:\n\t{:?}", self.next_step);
        println!("\treduced_code:\n\t{:?}", self.reduced_code);
        println!("\tdistance:\n\t{:?}", self.distance);
        println!("\tsingle_item_transposition_data:");
        self.print_single_item_transposition_data();
        println!("\tblock_slide_reduction_memo:");
        self.print_block_slide_reduction_memo();
        println!("\tvisited:");
        self.print_visited();
        println!("====================================================================================");
    }
    fn init_visited(size: usize) -> Vec<Vec<Vec<bool>>> {
        let mut visited: Vec<Vec<Vec<bool>>> = vec![];
        let lehmer_code_range = 0..constants::FACTORIALS[size as usize];
        for code in lehmer_code_range {
            let mut current_code: Vec<Vec<bool>> = vec![];
            for i in 1..size {
                let mut current_start_pos: Vec<bool> = vec![];
                for j in 0..size {
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
        let lehmer_code_range = 0..constants::FACTORIALS[size as usize];
        for code in lehmer_code_range {
            next_step.push(INVALID_PAIR);
        }
        next_step
    }

    fn init_distance(size: usize) -> Vec<u8> {
        let mut distance: Vec<u8> = vec![];
        let lehmer_code_range = constants::FACTORIALS[size as usize];
        for i in 0..lehmer_code_range {
            distance.push(99);
        }
        distance
    }

    fn init_single_item_transposition_data(size: usize) -> Vec<Vec<Vec<u32>>> {
        let mut single_item_transposition_memo = vec![];
        let lehmer_code_range = constants::FACTORIALS[size as usize];

        for code in 0..lehmer_code_range {
            let mut code_vector = vec![];
            for j in 0..size {
                let mut code_item_vector = vec![];
                for i in 0..j {
                    code_item_vector.push(
                        adjacency_calculator::get_code_shifting_item_to_new_position(
                            size, code, j, i as i8))
                }
                code_vector.push(code_item_vector);
            }
            single_item_transposition_memo.push(code_vector);
        }
        single_item_transposition_memo
    }

    fn init_is_pure(size: usize) -> Vec<(usize, u32)> {
        let mut is_pure = vec![];

        for code in 0..constants::FACTORIALS[size as usize] {
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
            if self.reduced_code[code as usize].0 == self.size {
                self.pure_permutations.push(code as u32);
            }
        }
    }

    pub fn init_block_slide_reduction_memo(size: usize) -> Vec<Vec<Vec<i64>>> {
        let mut block_slide_reduction_memo = vec![];
        let lehmer_code_range = constants::FACTORIALS[size as usize];

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
    //todo: don't have k, doesn't make sense for k
    // we are finding the lowest distance label if we slide block [i, j]
    // needs terminal conditions

    fn find_smallest_reducible_code_sliding_block_beyond(&mut self, cur_code: u32, i: usize, j: usize) -> i64 {
        let block_memo_value = self.block_slide_reduction_memo[cur_code as usize][i as usize][j as usize];
        if block_memo_value != UNDEFINED {
            return block_memo_value;
        } else {
            let mut min_size_i_j = 99;
            let mut min_code_i_j = 99;
            let mut min_dist_i_j = 99;
            //todo: convert this into a tail recursion loop for updating values correctly
            // propagate the min_values from here upwards
            let future_min_code: i64 =
                if j + 1 < self.size {
                    let new_code = adjacency_calculator::get_code_shifting_item_to_new_position(
                        self.size, cur_code, j + 1, i as i8 - 1);
                    self.find_smallest_reducible_code_sliding_block_beyond(new_code, i + 1, j + 1)
                } else { -1 };
            let cur_code_size = self.reduced_code[cur_code as usize].0;
            let cur_code_code = self.reduced_code[cur_code as usize].1;
            let cur_code_dist = self.distance[cur_code as usize];


            if future_min_code == -1 || cur_code_dist < self.distance[future_min_code as usize] {
                if self.reduced_code[cur_code as usize].0 < self.size {
                    self.update_block_slide_reduction_memo(i, j, cur_code.clone(), cur_code.clone());
                    if DEBUG_ENABLED {
                        println!("\nfind_smallest_reducible_code_sliding_block_beyond called");
                        println!("\tparams: cur_code: {}, i: {}, j: {} ", cur_code, i, j);
                        println!("returning : {}", cur_code);
                    }
                    return cur_code as i64;
                } else {
                    self.update_block_slide_reduction_memo(i, j, cur_code.clone(), cur_code.clone());
                    self.block_slide_reduction_memo[cur_code as usize][i as usize][j as usize] = -1;
                    return -1;
                }
            } else if self.reduced_code[future_min_code as usize].0 < self.size {
                return future_min_code;
            } else {
                return -1;
            }
        }
    }

    fn update_block_slide_reduction_memo(&mut self, i: usize, j: usize, cur_code: u32, new_code: u32) {
        if DEBUG_ENABLED {
            println!("update_block_slide_reduction_memo called with params\
            \n\tcode: {}, {:?} [ i : {}, j = {}]\
             \n\tnew_code: {}, {:?}",
                     cur_code,
                     permutation_label::get_permutation_from_lehmer_code(self.size, cur_code as usize),
                     i, j, new_code,
                     permutation_label::get_permutation_from_lehmer_code(self.size, new_code as usize));
        }

        if self.distance[cur_code as usize] > self.distance[new_code as usize] {
            self.block_slide_reduction_memo[cur_code as usize][i as usize][j as usize] = new_code as i64;
        }
    }

    //todo: min_size, min_code, min_dist could be packaged into single struct?
//todo: verify the inference of distance is not mistaken, i.e. no +-1 error when inheriting
    // this is forked
    pub fn process_block_slide_reduction_memo(&mut self, permutations_data: &Vec<PermutationsData>) {
        if DEBUG_ENABLED {
            println!("process_block_slide_reduction_memo called!");
        }
        let mut code_for_minimum_distance_reducible_permutation_reachable = 99;
        let mut distance_for_minimum_distance_reducible_permutation_reachable = 99;
        let mut size_for_minimum_distance_reducible_permutation_reachable = 99;
        for code in 0..constants::FACTORIALS[self.size as usize] {
            for i in 0..self.size {
                for j in i..self.size {
                    let min_code_in_block =
                        self.find_smallest_reducible_code_sliding_block_beyond(code, i, j);

                    if min_code_in_block >= 0 &&
                        self.distance[min_code_in_block as usize]
                            < distance_for_minimum_distance_reducible_permutation_reachable {
                        let reduced_code = self.reduced_code[min_code_in_block as usize];
                        size_for_minimum_distance_reducible_permutation_reachable = reduced_code.0 as u8;
                        code_for_minimum_distance_reducible_permutation_reachable = reduced_code.1;
                        if min_code_in_block as u32 != code {
                            distance_for_minimum_distance_reducible_permutation_reachable =
                                self.distance[min_code_in_block as usize] + 1;
                        } else {
                            distance_for_minimum_distance_reducible_permutation_reachable =
                                self.distance[min_code_in_block as usize];
                        }
                    }
                }
            }
            self.update_distance_and_next_step_for_code(
                code,
                distance_for_minimum_distance_reducible_permutation_reachable,
                size_for_minimum_distance_reducible_permutation_reachable,
                code_for_minimum_distance_reducible_permutation_reachable);
        }
    }

    pub fn calculate(&self) {}

    //Complexity: O(n!*n^2)?
    pub fn update_init_on_basis_of_previous(&mut self, permutations_data: &Vec<PermutationsData>) {
        //Complexity: O(n!)
        self.update_distance_and_next_step_for_reducible_permutations(permutations_data);
        self.print();
//        self.process_block_slide_reduction_memo(permutations_data);
        //Complexity: O(n!*n^2)
        self.update_distance_and_next_step_for_pure_permutations(permutations_data);
    }

    fn get_adjacent_reduced_permutation_to(&self, code: u32) -> Vec<u32> {
        let mut adjacent_reduced_codes = vec![];
        for i in 0..self.size-1 {
            for j in i..self.size-1 {
                let mut cur_code = code;
                let block_size =  (j - i + 1) as i8;
                for k in j+1..self.size {
                    cur_code = get_code_shifting_item_to_new_position(
                        self.size,
                        cur_code.clone(),
                        k,
                        k as i8 - block_size - 1);

                    if self.reduced_code[cur_code as usize].0 < self.size {
                        adjacent_reduced_codes.push(cur_code);
                    }
                }
            }
        }
        return adjacent_reduced_codes;
    }

    fn update_distance_and_next_step_for_pure_permutations(&mut self, permutations_data: &Vec<PermutationsData>) {
        for pos in 0..self.pure_permutations.len() {
            //todo: implement
            let code = self.pure_permutations[pos];
            let adjacent_reduced_permutations = self.get_adjacent_reduced_permutation_to(code as u32);

            if !DEBUG_ENABLED {
                println!("update_distance_and_next_step_for_pure_permutations called for\
                 \n\tsize: {} code : {} permutation: {:?}\
                \n\tadjacent: {:?}",
                         self.size, code,
                         permutation_label::get_permutation_from_lehmer_code(self.size, code as usize),
                         adjacent_reduced_permutations);
            }

            let mut min_dist = 99;
            let mut min_size = 99;
            let mut min_label = 99;

            for code in adjacent_reduced_permutations {
                let pair = self.reduced_code[code as usize];
                let new_size = pair.0;
                let new_label = pair.1;
                let new_dist = permutations_data[new_size as usize].distance[new_label as usize] + 1;

                if new_dist < min_dist {
                    min_dist = new_dist;
                    min_size = new_size;
                    min_label = new_label;
                }
            }
            self.update_distance_and_next_step_for_code(code as u32, min_dist, min_size as u8, min_label);
        }
    }

    fn update_distance_and_next_step_for_code(&mut self, code: u32, new_dist: u8, new_size: u8, new_code: u32) {
        self.distance[code as usize] = new_dist;
        self.next_step[code as usize] = (new_size, new_code);
        if DEBUG_ENABLED {
            println!("updating distance and next_step for code\
            \n\toriginal code : {}\
            \n\tUpdated : self.distance: {}, next_step: {:?}",
                     code,
                     self.distance[code as usize],
                     self.next_step[code as usize])
        }
    }

    fn update_distance_and_next_step_for_reducible_permutations(&mut self, permutations_data: &Vec<PermutationsData>) {
        if DEBUG_ENABLED {
            println!("update_distance_and_next_step_for_reducible_permutations called!\
            \n Params: size: {:?} ({:?})", self.size, self.reduced_code);
        }

        for code in 0..self.reduced_code.len() {
            let size_code_pair = self.reduced_code[code as usize];
            if size_code_pair.0 == self.size {
                continue;
            } else {
                let new_size = size_code_pair.0;
                let new_code = size_code_pair.1;
                if DEBUG_ENABLED {
                    println!("new size: {}, new_code: {}", new_size, new_code);
                }
                let new_dist = permutations_data[new_size as usize].distance[new_code as usize];
                self.update_distance_and_next_step_for_code(code as u32, new_dist, new_size as u8, new_code);
            }
        };
    }

    //todo: implement
    pub fn process_permutation(&mut self, code: u32) {
        if DEBUG_ENABLED {
            println!("process_permutation called for size : {}, permutation: {:?} ",
                     self.size,
                     permutation_label::get_permutation_from_lehmer_code(self.size, code as usize));
        }
        for i in 0..self.size - 1 {
            for j in i..self.size - 1 {
                if self.visited[code as usize][i][j] {
                    continue;
                }
                let block_size = (j - i) + 1;
                let mut new_code = code;
                self.visited[code as usize][i][j] = true;
                for k in j + 1..self.size {
                    // j+1 - j + i - 2
                    // i - 1
                    if DEBUG_ENABLED {
                        let i1 = k as i8 - block_size as i8 - 1;
                        println!("get_new_code for : {} {} {}", new_code, k, i1);
                    }
                    new_code = get_code_shifting_item_to_new_position(
                        self.size,
                        new_code,
                        k,
                        (k as i8 - block_size as i8 - 1) as i8);
                    //keystone to optimization
                    if self.visited[new_code as usize][k - block_size][k - 1] {
                        break;
                    }
                    self.visited[new_code as usize][k - block_size][k - 1] = true;
                    if self.distance[code as usize] < self.distance[new_code as usize] {
                        self.distance[new_code as usize] = self.distance[code as usize] + 1;
                        self.next_step[new_code as usize] = (self.size as u8, code);
                    }
                }
            }
        }
    }

    pub fn process_pure_permutations(&mut self) {
        if DEBUG_ENABLED {
            println!("Process pure_permutations called for size : {} ", self.size);
        }

        let mut unprocessed_batch = self.pure_permutations.clone();
        let mut batch_ct = 0;
        while !unprocessed_batch.is_empty() {
            batch_ct += 1;
            if DEBUG_ENABLED {
                println!("batch processing batch: {} \
                \n\tbatch has: {:?}", batch_ct, unprocessed_batch);
            }
            let mut current_max_dist = 0;
            let mut current_min_dist = 99;
            let mut next_batch = vec![];
            for pos in 0..unprocessed_batch.len() {
                let current_code = self.pure_permutations[pos];
                let current_code_dist = self.distance[current_code as usize];
                current_min_dist = min(current_min_dist, current_code_dist);
                current_max_dist = max(current_max_dist, current_code_dist);
            }

            if current_min_dist != 99 && current_min_dist + 1 >= current_max_dist {
                if DEBUG_ENABLED {
                    println!("batch processing stopped: \
                    \n\t min_dist: {}, max_dist: {}", current_min_dist, current_max_dist);
                }
                break;
            }
            for pos in 0..unprocessed_batch.len() {
                let current_code = self.pure_permutations[pos];
                let current_code_dist = self.distance[current_code as usize];
                if current_code_dist == current_min_dist {
                    next_batch.push(current_code);
                }
            }

            for item in next_batch {
                self.process_permutation(item);
            }

            unprocessed_batch.retain(|&x| self.visited[x as usize][0][0] == false);
        }
    }
}

pub fn generate_data_for_size_up_to(max_size: usize) -> Vec<PermutationsData> {
    let mut permutations_data = vec![
        PermutationsData {
            size: 0,
            visited: vec![],
            distance: vec![0],
            next_step: vec![(0, 0)],
            single_item_transposition_data: vec![],
//todo: fill?
            block_slide_reduction_memo: vec![],
            reduced_code: vec![(0, 0)],
            pure_permutations: vec![0],
        },
//        PermutationsData {
//            size: 1,
//            visited: vec![vec![vec![true]]],
//            distance: vec![0],
////todo: verify?
//            next_step: vec![(0, 0)],
//            single_item_transposition_data: vec![],
//            //todo: fill?
//            block_slide_reduction_memo: vec![],
//            reduced_code: vec![(0, 0)],
//            pure_permutations: vec![],
//        }
    ];
    for size in 1..max_size {
        let mut cur_size_permutations_data = PermutationsData::init(size);
        cur_size_permutations_data.update_init_on_basis_of_previous(&permutations_data);
        cur_size_permutations_data.process_pure_permutations();
        permutations_data.push(cur_size_permutations_data);
    }
    return permutations_data;
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

    #[test]
    fn test_generate_data() {
        let mut permutations_data = generate_data_for_size_up_to(5);
        for permutation_data in permutations_data {
            permutation_data.print();
        }
    }

    #[test]
    fn find_smallest_reducible_code_sliding_block_beyond() {}
}