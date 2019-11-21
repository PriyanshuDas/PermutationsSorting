use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;
use std::cmp::{min, max};
use crate::permutation::adjacency_calculator::{get_code_shifting_item_to_new_position, AdjacencyCalculator};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use rayon::prelude::*;

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
    adjacency_calculator: AdjacencyCalculator,
    reduced_code: Vec<(usize, u32)>,
    pure_permutations: Vec<u32>,
}


//todo: space complexity can be reduced by closely packing some memos
impl PermutationsData {
    fn print_visited(&self) {}

    pub fn print(&self) {
        println!("================================[Data for size = {}]================================", self.size);
        println!("\tsize: {:?}", self.size);
        println!("\tpure_permutations:\n\t{:?}", self.pure_permutations);
        println!("\tnext_step:\n\t{:?}", self.next_step);
        println!("\treduced_code:\n\t{:?}", self.reduced_code);
        println!("\tdistance:\n\t{:?}", self.distance);
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
        fn build_visited_vec_for_code(size: usize) -> Vec<Vec<bool>> {
            let mut current_code: Vec<Vec<bool>> = vec![];
            for i in 1..size {
                let mut current_start_pos: Vec<bool> = vec![];
                for j in 0..size {
                    current_start_pos.push(false);
                }
                current_code.push(current_start_pos);
            }
            current_code
        }

        let mut visited: Vec<Vec<Vec<bool>>> = vec![];
        (0..constants::FACTORIALS[size as usize]).collect::<Vec<u32>>()
            .par_iter_mut()
            .map(|pos| build_visited_vec_for_code(size))
            .collect()
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

    fn init_reduced_code(size: usize) -> Vec<(usize, u32)> {
        fn get_reduced_code(size: usize, code: u32) -> (usize, u32) {
            let permutation = permutation_label::get_permutation_from_lehmer_code(
                size, code as usize);
            let reduced_permutation = permutation_helper::reduce_permutation(&permutation);
            if reduced_permutation.len() == permutation.len() {
                let value_to_push = (size, code);
                return value_to_push;
            } else {
                let reduced_code = permutation_label::get_lehmer_code_from_permutation(&reduced_permutation);
                let value_to_push = (reduced_permutation.len(), reduced_code);
                return value_to_push;
            }
        }

        (0..constants::FACTORIALS[size as usize]).collect::<Vec<u32>>()
            .par_iter_mut()
            .map(|code| get_reduced_code(size, *code))
            .collect()
    }

    pub fn init_pure_permutations(&mut self) {
        for code in 0..self.reduced_code.len() {
            if self.reduced_code[code as usize].0 == self.size {
                self.pure_permutations.push(code as u32);
            }
        }
    }

    pub fn init(size: usize) -> PermutationsData {
        let now = Instant::now();
        let mut prev_duration = now.elapsed();
        let mut cur_duration = now.elapsed();
        let next_step = PermutationsData::init_next_step(size);
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_next_step");
            prev_duration = cur_duration;
        }

        let visited = PermutationsData::init_visited(size);
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_visited");
            prev_duration = cur_duration;
        }

        let distance = PermutationsData::init_distance(size);
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_distance");
            prev_duration = cur_duration;
        }

        let adjacency_calculator = AdjacencyCalculator::init(size);
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "adjacency_calculator::init()");
            prev_duration = cur_duration;
        }

        let reduced_code = PermutationsData::init_reduced_code(size);
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "reduced_code");
            prev_duration = cur_duration;
        }

        let mut permutations_data = PermutationsData {
            size,
            next_step,
            visited,
            distance,
            adjacency_calculator,
            pure_permutations: vec![],
            reduced_code,
        };

        permutations_data.init_pure_permutations();
        if !DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_pure_permutations");
            prev_duration = cur_duration;
        }

        return permutations_data;
    }

    fn log_time_taken(prev_duration: &mut Duration, cur_duration: &mut Duration, name: &str) {
        let time_taken_ms = cur_duration.as_millis() - prev_duration.as_millis();
        let time_taken_s = cur_duration.as_secs() - prev_duration.as_secs();
        println!("{} took : {} ms, {} s", name, time_taken_ms, time_taken_s);
    }

    //todo: write tests
    //todo: don't have k, doesn't make sense for k
    // we are finding the lowest distance label if we slide block [i, j]
    // needs terminal conditions

    pub fn calculate(&self) {}

    //Complexity: O(n!*n^2)?
    pub fn update_init_on_basis_of_previous(&mut self, permutations_data: &Vec<PermutationsData>) {
        //Complexity: O(n!)
        self.update_distance_and_next_step_for_reducible_permutations(permutations_data);
        //Complexity: O(n!*n^3)
//        self.update_distance_and_next_step_for_pure_permutations(permutations_data);
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
                    let new_pos_for_optimized = k as i8 - block_size as i8 - 1;
                    let delta_for_move = self.adjacency_calculator.get_delta_for_moving_item(
                        cur_code, k, new_pos_for_optimized);

                    cur_code = (cur_code as i32 + delta_for_move) as u32;

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


    fn update_distance_and_next_step_for_pure_permutations_optimized(&mut self, permutations_data: &Vec<PermutationsData>) {
        let mut reduced_codes: Vec<(u32, u8)> =
            (0..constants::FACTORIALS[self.size]).collect::<Vec<u32>>()
                .par_iter()
                .filter(|code| self.reduced_code[**code as usize].0 < self.size)
                .map(|code| (*code, self.distance[*code as usize]))
                .collect();

        reduced_codes.sort_by(|a, b| a.1.cmp(&b.1));
        return;
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
//        let n = self.size;
//        self.distance = self.reduced_code.par_iter()
//            .map(|reduced_size_code_pair|
//                if reduced_size_code_pair.0 != n { reduced_size_code_pair.1 as u8 } else { 99 as u8 }
//            ).collect();
//
//        self.next_step = self.reduced_code.par_iter()
//            .map(|reduced_size_code_pair|
//                if reduced_size_code_pair.0 != n {
//                    (reduced_size_code_pair.0 as u8, reduced_size_code_pair.1 as u32)
//                } else { (n as u8, 99 as u32) }
//            ).collect();
        for code in 0..self.reduced_code.len() {
            let size_code_pair = self.reduced_code[code as usize];
            if size_code_pair.0 == self.size {
                continue;
            } else {
                let new_size = size_code_pair.0;
                let new_code = size_code_pair.1;
                if DEBUG_ENABLED {
                    println!("new size: {}, new_code: {}", new_size, new_code);
                    println!("permutations_data_lenght: {}", permutations_data.len());
                }
                let new_dist = permutations_data[new_size as usize].distance[new_code as usize];
                self.distance[code as usize] = new_dist;
                self.next_step[code as usize] = (new_size as u8, new_code);
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
                    let new_pos_for_optimized = k as i8 - block_size as i8 - 1;
                    let delta_for_move = self.adjacency_calculator.get_delta_for_moving_item(
                        cur_code, k, new_pos_for_optimized);
                    cur_code = (cur_code as i32 + delta_for_move) as u32;

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
    pub fn process_all_permutations(&mut self) {
        let mut next_batch = vec![0];
        while next_batch.len() > 0 {
            let current_batch = next_batch.clone();
            let mut next_batch = vec![];
            current_batch.par_iter()
                .map(|code| process_permutation_adjacency())
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
            adjacency_calculator: AdjacencyCalculator::init(0),
//todo: fill?
            reduced_code: vec![(0, 0)],
            pure_permutations: vec![0],
        },
    ];
    for size in 1..max_size {
        let now = Instant::now();
        let mut cur_size_permutations_data = PermutationsData::init(size);
        let elapsed_on_init = now.elapsed();
        if !DEBUG_ENABLED {
            println!("For size {}, init time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_init.as_millis(),
                     elapsed_on_init.as_secs());
        }
        cur_size_permutations_data.update_init_on_basis_of_previous(&permutations_data);
        let elapsed_on_update_init = now.elapsed();
        if !DEBUG_ENABLED {
            println!("For size {}, update_init_on_basis_of_previous time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_update_init.as_millis() - elapsed_on_init.as_millis(),
                     elapsed_on_update_init.as_secs() - elapsed_on_init.as_secs());
        }
        cur_size_permutations_data.process_all_permutations();
        let elapsed_on_process_pure_permutations = now.elapsed();
        if !DEBUG_ENABLED {
            println!("For size {}, process_pure_permutations: time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_process_pure_permutations.as_millis() - elapsed_on_update_init.as_millis(),
                     elapsed_on_process_pure_permutations.as_secs() - elapsed_on_update_init.as_secs());
        }
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


    //todo: for high sizes takes too long.
    #[test]
    fn test_generate_data() {
        let mut permutations_data = generate_data_for_size_up_to(10);
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