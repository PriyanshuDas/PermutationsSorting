use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;
use std::cmp::{min, max};
use crate::permutation::adjacency_calculator::{get_code_shifting_item_to_new_position, AdjacencyCalculator};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use rayon::prelude::*;
use crate::permutation::distance_calculator::ProcessingStatus::{UNPROCESSED, PROCESSED};
use std::fmt;
use std::fmt::{Formatter, Error};

const DEBUG_ENABLED: bool = false;
const INVALID_PAIR: (u8, u32) = (99, 99);
const UNDEFINED: i64 = -2;

//todo: this is getting extremely complicated, streamline this process!

//not working
// next_step
// distance
#[derive(PartialEq)]
#[derive(Debug)]
pub enum ProcessingStatus {
    UNPROCESSED,
    PROCESSING,
    PROCESSED,
}


pub struct PermutationsData {
    //    visited[code][i][j] => perm_code 's block [i,j] has been visited before
    size: usize,
    visited: Vec<Vec<Vec<ProcessingStatus>>>,
    distance: Vec<u8>,
    //next_step: size, code
    next_step: Vec<(u8, u32)>,
    //[code][i][j] => j moved to after i in code => new code
    adjacency_calculator: AdjacencyCalculator,
    reduced_code: Vec<(usize, u32)>,
    pure_permutations: Vec<u32>,
}


fn get_all_blocks_for_size(size: usize) -> Vec<(usize, usize)> {
    let mut pairs = vec![];
    for i in 0..size - 1 {
        for j in i..size - 1 {
            pairs.push((i, j));
        }
    };
//    println!("{:?}", pairs);
    pairs
}

//todo: space complexity can be reduced by closely packing some memos
impl PermutationsData {
    fn print_visited(&self) {}

    pub fn print(&self) {
        println!("================================[Data for size = {}]================================", self.size);
//        println!("\tsize: {:?}", self.size);
//        println!("\tpure_permutations:\n\t{:?}", self.pure_permutations);
//        println!("\tnext_step:\n\t{:?}", self.next_step);
//        println!("\treduced_code:\n\t{:?}", self.reduced_code);
//        println!("\tdistance:\n\t{:?}", self.distance);
//        println!("\tvisited:");
//        self.print_visited();

        println!("code\tpermutation\t\tdist\tnext_step\tnew_permutation");
        for code in &self.pure_permutations {
            let permutation = permutation_label::get_permutation_from_lehmer_code(self.size, *code as usize);
            let dist = self.distance[*code as usize];
            let next_step = self.next_step[*code as usize];

            let new_perm = if next_step.0 < 99 {
                permutation_label::get_permutation_from_lehmer_code(
                    next_step.0 as usize, next_step.1 as usize)
            } else { vec![] };

            println!("{}\t{:?}\t\t{}\t{:?}\t{:?}", *code, permutation, dist, next_step, new_perm);
        }
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
                .or_insert_with(|| 1);
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

    fn init_visited(size: usize) -> Vec<Vec<Vec<ProcessingStatus>>> {
        fn build_visited_vec_for_code(size: usize) -> Vec<Vec<ProcessingStatus>> {
            let mut current_code: Vec<Vec<ProcessingStatus>> = vec![];
            for i in 0..size {
                let mut current_start_pos: Vec<ProcessingStatus> = vec![];
                for j in 0..size {
                    current_start_pos.push(UNPROCESSED);
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
        next_step[0] = (0, 0);
        next_step
    }

    fn init_distance(size: usize) -> Vec<u8> {
        let mut distance: Vec<u8> = vec![];
        let lehmer_code_range = constants::FACTORIALS[size as usize];
        for i in 0..lehmer_code_range {
            distance.push(99);
        }
        distance[0] = 0;
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
        if DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_next_step");
            prev_duration = cur_duration;
        }

        let visited = PermutationsData::init_visited(size);
        if DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_visited");
            prev_duration = cur_duration;
        }

        let distance = PermutationsData::init_distance(size);
        if DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "init_distance");
            prev_duration = cur_duration;
        }

        let adjacency_calculator = AdjacencyCalculator::init(size);
        if DEBUG_ENABLED {
            cur_duration = now.elapsed();
            PermutationsData::log_time_taken(&mut prev_duration, &mut cur_duration, "adjacency_calculator::init()");
            prev_duration = cur_duration;
        }

        let reduced_code = PermutationsData::init_reduced_code(size);
        if DEBUG_ENABLED {
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
        if DEBUG_ENABLED {
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

            //this is currently taking O(n^3), but ideally should be O(n^2)
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

    pub fn process_permutation(&mut self, code: u32) {
        if DEBUG_ENABLED {
            println!("process_permutation called for size : {}, permutation: {} ",
                     self.size,
                     code);
        }
        for i in 0..self.size - 1 {
            for j in i..self.size - 1 {
                if self.visited[code as usize][i][j] != UNPROCESSED {
                    continue;
                }
                let block_size = (j - i) + 1;
                let mut cur_code = code;
                self.visited[code as usize][i][j] = PROCESSED;
                for k in j + 1..self.size {
                    let new_pos_for_optimized = k as i8 - block_size as i8 - 1;
                    let delta_for_move = self.adjacency_calculator.get_delta_for_moving_item(
                        cur_code, k, new_pos_for_optimized);
                    cur_code = (cur_code as i32 + delta_for_move) as u32;

                    if self.visited[cur_code as usize][k - block_size][k - 1] != UNPROCESSED {
                        break;
                    }
                    self.visited[cur_code as usize][k - block_size][k - 1] = PROCESSED;
                    if self.distance[code as usize] < self.distance[cur_code as usize] {
                        self.distance[cur_code as usize] = self.distance[code as usize] + 1;
                        self.next_step[cur_code as usize] = (self.size as u8, code);
                    }
                }
            }
        }

        //todo: should not be needed
        self.visited[code as usize][0][0] = PROCESSED;
    }


    pub fn process_permutation_paralleled(&mut self, code: u32) -> Vec<u32> {
        fn process_block_move_for_code(permutation_data: &PermutationsData, code: u32, i: usize, j: usize) ->
        (Vec<u32>, Vec<(u32, usize, usize)>) {
            let block_size = (j - i) + 1;
            let mut cur_code = code;
            let mut adjacent_codes: Vec<u32> = vec![];
            let mut visited_tuples: Vec<(u32, usize, usize)> = vec![];
            for k in j + 1..permutation_data.size {
                let new_pos_for_optimized = k as i8 - block_size as i8 - 1;
                let delta_for_move = permutation_data.adjacency_calculator.get_delta_for_moving_item(
                    cur_code, k, new_pos_for_optimized);
                cur_code = (cur_code as i32 + delta_for_move) as u32;

                let block_shift = k - j;
                let new_i = i + block_shift;
                let new_j = j + block_shift;

                if DEBUG_ENABLED {
                    println!("@{} code : {}, block : ({}, {}), k : {},\
                 \n\tnew_code: {}, new_block: ({}, {})\
                 \n\tVISITED: {:?}", permutation_data.size,
                             code, i, j, k, cur_code, new_i, new_j,
                             permutation_data.visited[cur_code as usize][new_i][new_j]);
                }

                if permutation_data.visited[cur_code as usize][new_i][new_j] != UNPROCESSED {
                    break;
                }
                visited_tuples.push((cur_code, new_i, new_j));
                if permutation_data.distance[code as usize] < permutation_data.distance[cur_code as usize] {
                    adjacent_codes.push(cur_code);
                }
            }
            return (adjacent_codes, visited_tuples);
        }

        let dist = self.distance[code as usize];

        let updatable_items: Vec<(Vec<u32>, Vec<(u32, usize, usize)>)> =
            get_all_blocks_for_size(self.size).par_iter_mut()
                .map(|range| {
//                println!("Woww! {:?}", range);
                    process_block_move_for_code(&self, code, range.0, range.1)
//                return (vec![], vec![]);
                }).collect();

        let mut next_batch_items = vec![];
        if DEBUG_ENABLED
        {
            println!("for code: {}", code);
            println!("{:?}", updatable_items);
        }

        for item in updatable_items {
            let adjacent_codes: Vec<u32> = item.0;
            let visited_tuples: Vec<(u32, usize, usize)> = item.1;
            for code in adjacent_codes {
                self.distance[code as usize] = dist + 1;
                next_batch_items.push(code);
            }

            for tuple in visited_tuples {
                self.visited[tuple.0 as usize][tuple.1][tuple.2] = PROCESSED;
            }
        }
        if DEBUG_ENABLED {
            println!("Next_Batch : {:?}", next_batch_items);
        }
        next_batch_items
    }


    // Complexity: O(n!*n^2) time and O(n!) space
    pub fn process_all_permutations(&mut self) {
        let mut next_batch = vec![0];
        while next_batch.len() > 0 {
            let mut current_batch_non_reducible = vec![];
            let mut current_batch_reducible = vec![];
            let mut cur_dist = self.distance[next_batch[0] as usize];
            let mut max_dist = 0;

            for item in &next_batch {
                if self.next_step[*item as usize].0 == self.size as u8 {
                    current_batch_non_reducible.push((*item).clone());
                } else if self.next_step[*item as usize].0 < self.size as u8 {
                    current_batch_reducible.push((*item).clone());
                }
            }
            for dist in &(self.distance) {
                max_dist = max(*dist, max_dist);
            }

            if DEBUG_ENABLED {
                println!("Size of upcoming batch : {}", next_batch.len());
                println!("Max Distance : {}", max_dist);
                println!("Current Distance : {}", cur_dist);
            }

            let now = Instant::now();
            let mut old_time = now.elapsed();
            next_batch = vec![];

            if cur_dist as i8 > max_dist as i8 - 2 {
                println!("Distance of upcoming batch : {}", cur_dist);
                println!("Max Distance : {}", max_dist);
                break;
            }

            for code in current_batch_reducible {
                self.process_permutation_paralleled(code)
                    .iter()
                    .for_each(|code| next_batch.push(*code));
                if !DEBUG_ENABLED {
                    println!("Processed: {}", code);
                }
            }

            max_dist = 0;

            for dist in &(self.distance) {
                max_dist = max(*dist, max_dist);
            }

            if DEBUG_ENABLED {
                println!("Max Distance For Irreducible : {}", max_dist);
            }
            if cur_dist as i8 > max_dist as i8 - 2 {
                println!("Distance of upcoming batch : {}", cur_dist);
                println!("Max Distance : {}", max_dist);
                break;
            }
            for code in current_batch_non_reducible {
                self.process_permutation_paralleled(code)
                    .iter()
                    .for_each(|code| next_batch.push(*code));
                if DEBUG_ENABLED {
                    println!("Processed: {}", code);
                    println!("next_batch size : {:?}", next_batch.len());
                }
            }

            if DEBUG_ENABLED {
                let current_time = now.elapsed();
                println!("time taken to process : {:?} ms",
                         current_time.as_millis() - old_time.as_millis());
            }
        }
        if DEBUG_ENABLED {
            println!("Processed!");
        }
    }

    //todo: implement cleanly

    fn set_initial_distance_for_pure_permutations(&mut self) {
        fn get_all_updates_for_block_movement_for_code(size: usize, code: usize,
                                                       i: usize, j: usize, distance_memo: &HashMap<(usize, usize, usize), usize>,
                                                       permutation_data: &PermutationsData) -> Vec<((usize, usize, usize), usize)> {
            let block_size = (j - i) + 1;
            let mut cur_code = code;

            //(code, i, j) -> value
            let mut updated_values: Vec<((usize, usize, usize), usize)> = vec![];
            updated_values.push(((code, i, j), code));
            for k in j + 1..size {
                let new_pos_for_optimized = k as i8 - block_size as i8 - 1;

                let delta_for_move = permutation_data.adjacency_calculator
                    .get_delta_for_moving_item(cur_code as u32, k, new_pos_for_optimized);

                cur_code = (cur_code as i32 + delta_for_move) as usize;

                let block_shift = k - j;
                let new_i = i + block_shift;
                let new_j = j + block_shift;
                //todo: is the key working as expected?
                let key = (cur_code, new_i, new_j);
                if distance_memo.contains_key(&key) {
                    updated_values.push((key, *distance_memo.get(&key).unwrap()));
                    break;
                } else {
                    updated_values.push(((key), cur_code));
                }
            }
            let n = updated_values.len();
            for pos in 1..n {
                if permutation_data.distance[updated_values[n - 1 - pos].1]
                    > permutation_data.distance[updated_values[n - pos].1] {
                    updated_values[n - 1 - pos].1 = updated_values[n - pos].1;
                }
            }
            updated_values
        }

        fn update_pure_code_adjacent_to_reducible(
            size: usize, code: usize,
            distance_memo: &mut HashMap<(usize, usize, usize), usize>,
            permutation_data: &mut PermutationsData) {
            let updatable_values: Vec<((usize, usize, usize), usize)> =
                get_all_blocks_for_size(size as usize)
                    .par_iter_mut()
                    .map(|block_pair| get_all_updates_for_block_movement_for_code(
                        size,
                        code,
                        block_pair.0,
                        block_pair.1,
                        distance_memo, permutation_data))
                    .flatten()
                    .collect();
            let mut min_adjacent_code = code;
            let mut min_adjacent_dist = 99;
            for pair in updatable_values {
                if !distance_memo.contains_key(&pair.0) {
                    distance_memo.insert(pair.0, pair.1);
                }
                if permutation_data.distance[pair.1] < min_adjacent_dist {
                    min_adjacent_code = pair.1;
                    if min_adjacent_code != code {
                        min_adjacent_dist = permutation_data.distance[min_adjacent_code] + 1;
                    } else {
                        min_adjacent_dist = permutation_data.distance[min_adjacent_code];
                    }
                }
            }

            if DEBUG_ENABLED {
                println!("for code: {}, min_adjacent_dist: {}, min_adjacent_code : {}",
                         code, min_adjacent_dist, min_adjacent_code);
            }

            permutation_data.distance[code] = min_adjacent_dist;
            permutation_data.next_step[code] = (size as u8, min_adjacent_code as u32);
            return;
        }

        let mut lowest_distance_map: HashMap<(usize, usize, usize), usize> = HashMap::new();
        for code in 0..constants::FACTORIALS[self.size] {
            update_pure_code_adjacent_to_reducible(
                self.size,
                code as usize, &mut lowest_distance_map, self)
        }
        if DEBUG_ENABLED {
            println!("{:?}", lowest_distance_map);
        }
        return;
    }

    fn process_pure_permutations(&mut self) {
        let mut unprocessed_codes: Vec<usize> =
            (0..constants::FACTORIALS[self.size] as usize).collect::<Vec<usize>>()
                .par_iter_mut()
                .filter(|code| self.reduced_code[**code].0 == self.size)
                .map(|code| *code)
                .collect();

        let mut next_batch_dist = 99;
        let mut max_unprocessed_dist = 0;

        for code in &unprocessed_codes {
            next_batch_dist = min(next_batch_dist, self.distance[*code]);
            max_unprocessed_dist = max(max_unprocessed_dist, self.distance[*code]);
        }

        let mut next_batch: Vec<usize> = unprocessed_codes
            .par_iter_mut()
            .filter(|code| self.distance[**code] == next_batch_dist)
            .map(|code| *code)
            .collect();

        while !next_batch.is_empty() {
            let cur_batch = next_batch.clone();

            if next_batch_dist + 2 > max_unprocessed_dist {
                break;
            }
            for code in cur_batch {
                self.process_permutation_paralleled(code as u32)
                    .iter()
                    .for_each(|code| next_batch.push(*code as usize));
                if DEBUG_ENABLED {
                    println!("Processed: {}", code);
                }
            }

            next_batch_dist = next_batch_dist + 1;
            for code in &unprocessed_codes {
                max_unprocessed_dist = max(max_unprocessed_dist, self.distance[*code]);
            }
        }
    }

    pub fn process_all_permutations_time_optimized(&mut self) {
        let now: Instant = Instant::now();
        let mut prev_time = now.elapsed();
//        self.set_initial_distance_for_pure_permutations();
        if !DEBUG_ENABLED {
            let mut new_time = now.elapsed();
            println!("set_initial_distance_for_pure_permutations, time taken: {} ms",
                     new_time.as_millis() - prev_time.as_millis());
            prev_time = new_time;
        }
        self.process_pure_permutations();
        if !DEBUG_ENABLED {
            let mut new_time = now.elapsed();
            println!("process_pure_permutations, time taken: {} ms",
                     new_time.as_millis() - prev_time.as_millis());
            prev_time = new_time;
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
        if !DEBUG_ENABLED {
            println!("Generating for size: {}", size);
        }
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
        cur_size_permutations_data.process_all_permutations_time_optimized();
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
//            println!("========================= Size = {} =========================", size);
            permutation_data.print_summary();
//            permutation_data.print();
//            println!("=============================================================");
        }
    }

    #[test]
    fn test_generate_data_5() {
        let mut permutations_data = generate_data_for_size_up_to(8);
        let expected_average: Vec<f32> = vec![0.0, 0.0, 1.0, 2.0, 2.125, 2.6944444, 3.1965065, 3.7095385];

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
//            permutation_data.print_summary();
        }
        assert_eq!(f32::abs(expected_average[7] - actual_stats[7].2) < 0.01, true);
    }

    #[test]
    fn find_smallest_reducible_code_sliding_block_beyond() {}
}