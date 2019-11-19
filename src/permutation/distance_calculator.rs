use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;
use std::cmp::{min, max};
use crate::permutation::adjacency_calculator::get_code_shifting_item_to_new_position;
use std::collections::HashMap;
use std::time::Instant;

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

    pub fn print_summary(&self) {
        let size = self.pure_permutations.len();
//            self.print();
        let mut total_distance: u64 = 0;
        let mut distance_distribution: HashMap<u8, Vec<u32>> = HashMap::new();
        let mut distance_counts: HashMap<u8, u32> = HashMap::new();
        for pos in 0..size {
            let code = self.pure_permutations[pos];
            let distance = self.distance[code as usize];
            total_distance += distance as u64;

            distance_distribution.entry(distance)
                .and_modify(|list| list.push(code))
                .or_insert_with(|| vec![]);

            distance_counts.entry(distance)
                .and_modify(|ct| *ct += 1)
                .or_insert_with(|| 0);
        }
        let average_distance = total_distance as f32 / size as f32;
        println!("size: {}, total_pures: {}, total_distance: {}, average_distance: {}\
            \n\t distance_distribution: {:?}"
                 , self.size,
                 size,
                 total_distance,
                 average_distance,
                 distance_counts);
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

    //todo: Optimize to O(n!*n^2 space and time complexity!)
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
                            size, code, j, (i as i8 -1) as i8))
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

    pub fn calculate(&self) {}

    //Complexity: O(n!*n^2)?
    pub fn update_init_on_basis_of_previous(&mut self, permutations_data: &Vec<PermutationsData>) {
        //Complexity: O(n!)
        self.update_distance_and_next_step_for_reducible_permutations(permutations_data);
        //Complexity: O(n!*n^2)
        self.update_distance_and_next_step_for_pure_permutations(permutations_data);
    }

    //todo: optimize with memoization
    fn get_adjacent_reduced_permutation_to(&self, code: u32) -> Vec<u32> {
        let mut adjacent_reduced_codes = vec![];
        for i in 0..self.size - 1 {
            for j in i..self.size - 1 {
                let mut cur_code = code;
                let block_size = (j - i + 1) as i8;
                for k in j + 1..self.size {
                    let new_pos = k as i8 - block_size - 1;
                    let new_pos_for_optimized = k - block_size as usize;
                    cur_code =
                        self.single_item_transposition_data[cur_code as usize][k][(new_pos_for_optimized)];
//                    cur_code = get_code_shifting_item_to_new_position(
//                        self.size,
//                        cur_code.clone(),
//                        k,
//                        new_pos);

//                    if !DEBUG_ENABLED && cur_code != cur_code_optimized {
//                        let original_perm =
//                            permutation_label::get_permutation_from_lehmer_code(
//                                self.size, code as usize);
//
//                        let cur_perm =
//                            permutation_label::get_permutation_from_lehmer_code(
//                                self.size, cur_code as usize);
//
//                        let new_perm =
//                            permutation_label::get_permutation_from_lehmer_code(
//                                self.size, cur_code_optimized as usize);
//
//                        println!("(get_adjacent_reduced_permutation_to {:?}\
//                         \n\tmoving ({} {} {})\
//                          \n\t correct new_perms: {:?}, new_pos: {}\
//                          \n\t optimized {:?}, new_pos: {}",
//                           original_perm, i, j, k, cur_perm, new_pos, new_perm, new_pos_for_optimized);
//                    }

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

            //this is currently taking O(n^4), but ideally should be O(n^2)
            let adjacent_reduced_permutations =
                self.get_adjacent_reduced_permutation_to(code as u32);

            if DEBUG_ENABLED {
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

    /*
        mutex locks might be required for:
        - self.visited[code][i][j]
        - self.visited[new_code][i'][j']

        Complexity: O(n^2) + part of O(n!) [for all process permutation called ever]

        todo: make as thread-safe as possible
    */
    pub fn process_permutation(&mut self, code: u32) {
        if DEBUG_ENABLED {
            println!("process_permutation called for size : {}, permutation: {} ",
                     self.size,
                     code);
        }
        for i in 0..self.size - 1 {
            for j in i..self.size - 1 {
                if self.visited[code as usize][i][j] {
                    continue;
                }
                let block_size = (j - i) + 1;
                let mut cur_code = code;
                self.visited[code as usize][i][j] = true;
                for k in j + 1..self.size {
                    let new_pos_for_optimized = k - block_size as usize;
                    cur_code =
                        self.single_item_transposition_data[cur_code as usize][k][(new_pos_for_optimized)];
                    if self.visited[cur_code as usize][k - block_size][k - 1] {
                        break;
                    }
                    self.visited[cur_code as usize][k - block_size][k - 1] = true;
                    if self.distance[code as usize] < self.distance[cur_code as usize] {
                        self.distance[cur_code as usize] = self.distance[code as usize] + 1;
                        self.next_step[cur_code as usize] = (self.size as u8, code);
                    }
                }
            }
        }

        //todo: should not be needed
        self.visited[code as usize][0][0] = true;
    }


    // Complexity: O(n!*n^2) time and O(n!) space
    pub fn process_pure_permutations(&mut self) {
        if DEBUG_ENABLED {
            println!("Process pure_permutations called for size : {} ", self.size);
        }

        let mut unprocessed_batch = self.pure_permutations.clone();
        let mut batch_ct = 0;

        //todo: this batch_calculation is wonky, fix?
        while !unprocessed_batch.is_empty() {
            batch_ct += 1;
            let mut current_max_dist = 0;
            let mut current_min_dist = 99;
            let mut next_batch = vec![];
            for pos in 0..unprocessed_batch.len() {
                let current_code = unprocessed_batch[pos];
                let current_code_dist = self.distance[current_code as usize];
                current_min_dist = min(current_min_dist, current_code_dist);
                current_max_dist = max(current_max_dist, current_code_dist);
            }
            if DEBUG_ENABLED {
                println!("batch: \
                    \n\t min_dist: {}, max_dist: {}", current_min_dist, current_max_dist);
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

            for item_pos in 0..next_batch.len() {
                self.process_permutation(next_batch[item_pos]);
            }

            if DEBUG_ENABLED {
                println!("batch processing for size: {}, batch: {} \
                \n\tbatch has: {:?}", self.size, batch_ct, unprocessed_batch);
                println!("processed_batch: {:?}", next_batch);
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
        let now = Instant::now();
        let mut cur_size_permutations_data = PermutationsData::init(size);
        let elapsed_on_init = now.elapsed();
        println!("For size {}, init time taken (ms) = {:?}, (s) = {:?}",
                 size,
                 elapsed_on_init.as_millis(),
                 elapsed_on_init.as_secs());
        cur_size_permutations_data.update_init_on_basis_of_previous(&permutations_data);
        let elapsed_on_update_init = now.elapsed();
        println!("For size {}, update_init_on_basis_of_previous time taken (ms) = {:?}, (s) = {:?}",
                 size,
                 elapsed_on_update_init.as_millis() - elapsed_on_init.as_millis(),
                 elapsed_on_update_init.as_secs() - elapsed_on_init.as_secs());
        cur_size_permutations_data.process_pure_permutations();
        let elapsed_on_process_pure_permutations = now.elapsed();
        println!("For size {}, process_pure_permutations: time taken (ms) = {:?}, (s) = {:?}",
                 size,
                 elapsed_on_process_pure_permutations.as_millis() - elapsed_on_update_init.as_millis(),
                 elapsed_on_process_pure_permutations.as_secs() - elapsed_on_update_init.as_secs());
        permutations_data.push(cur_size_permutations_data);
    }
    return permutations_data;
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
        let mut permutations_data = generate_data_for_size_up_to(9);
//        let expected_average = [];

        let mut actual_stats = vec![];
        for permutation_data in permutations_data {
            let size = permutation_data.pure_permutations.len();
            let mut total_distance: u64 = 0;
            let mut distance_distribution: HashMap<u8, Vec<u32>> = HashMap::new();
            let mut distance_counts: HashMap<u8, u32> = HashMap::new();
            for pos in 0..size {
                let code = permutation_data.pure_permutations[pos];
                let distance = permutation_data.distance[code as usize];
                total_distance += distance as u64;

                distance_distribution.entry(distance)
                    .and_modify(|list| list.push(code))
                    .or_insert_with(|| vec![]);

                distance_counts.entry(distance)
                    .and_modify(|ct| *ct += 1)
                    .or_insert_with(|| 0);
            }
            let average_distance = total_distance as f32 / size as f32;
            actual_stats.push((total_distance, size, average_distance));
            permutation_data.print_summary();
        }
    }

    #[test]
    fn find_smallest_reducible_code_sliding_block_beyond() {}
}