use crate::permutation::adjacency_calculator;
use crate::permutation::constants;
use crate::permutation::permutation_label;
use crate::permutation::permutation_helper;
use std::cmp::{min, max};
use crate::permutation::adjacency_calculator::{get_code_shifting_item_to_new_position, AdjacencyCalculator};
use std::collections::HashMap;
use std::time::{Instant, Duration};

use rayon::prelude::*;
use std::fmt;
use std::fmt::{Formatter, Error};
use std::ops::Range;
use crate::permutation::distance_calculator::distance_calculator_mem_efficient::ProcessingStatus::{UNPROCESSED, PROCESSING, PROCESSED};
use std::sync::Mutex;

// Time Complexity: O(n!*n^2 / t)
// Space Complexity: O(n!*n)

// Where t is number of concurrent threads
const DEBUG_ENABLED: bool = false;


//todo: handle clean mutex read/write, maybe use a method.
// implement mutex cleanly for processing_status and end_mask

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub enum ProcessingStatus {
    UNPROCESSED,
    PROCESSING,
    PROCESSED,
}

pub struct PermutationsData {
    size: usize,
    distance: Vec<Mutex<u8>>,
    adjacency_calculator: AdjacencyCalculator,
    reduced_code: Vec<(usize, u32)>,
    processing_status: Vec<Mutex<ProcessingStatus>>,
    end_mask: Vec<Vec<Mutex<u16>>>,
}


fn get_all_blocks_for_size(size: usize) -> Vec<(usize, usize)> {
    let mut pairs = vec![];
    for i in 0..size - 1 {
        for j in i..size - 1 {
            pairs.push((i, j));
        }
    };
    pairs
}


//todo: space complexity can be reduced by closely packing some memos
impl PermutationsData {
    fn print(&self) {
        println!("\t size: {:?}", self.size);
//        println!("\t distance: {:?}", self.distance);
//        println!("\t adjacency_calculator: {:?}", self.adjacency_calculator);
        println!("\t reduced_code: {:?}", self.reduced_code);
        println!("\t end_mask: {:?}", self.end_mask);
    }

    pub fn print_summary(&self) {
        println!("For size : {}", self.size);
        let pure_permutations: Vec<u32> = self.reduced_code.par_iter()
            .filter(|size_reduced_pair| size_reduced_pair.0 == self.size)
            .map(|pair| pair.1)
            .collect();
        let mut total_steps: u32 = pure_permutations.par_iter()
            .map(|code| self.get_distance(*code as usize) as u32)
            .sum();
        let mut total_count = pure_permutations.len();
        let average_dist = total_steps as f32 / (total_count as f32);
        if DEBUG_ENABLED {
            println!("\tPure Codes : {:?}", pure_permutations);
            println!("\tPure Codes size: {}", pure_permutations.len());
            println!("\tTotal Distance : {}", total_steps);
            let distance: Vec<u8> = self.distance.par_iter().map(|item| *(item.lock().unwrap())).collect();
            println!("\tDistance: {:?}", distance);
        }
        println!("\taverage_distance: {}", average_dist);
    }

    fn get_distance(&self, code: usize) -> u8 {
        *(self.distance[code].lock().unwrap())
    }

    fn get_max_dist(&self) -> u8 {
        let value = self.distance.par_iter()
            .map(|mutex| *(mutex.lock().unwrap()))
            .max();
        match value {
            Some(x) => x,
            None => 0
        }
    }

    fn set_distance(&mut self, code: usize, dist: u8) {
        let mut mutex_guard = self.distance[code].lock().unwrap();
        *mutex_guard = dist;
    }

    fn get_end_mask_bit(&self, code: usize, start_pos: usize, end_pos: usize) -> bool {
        return *(self.end_mask[code][start_pos].lock().unwrap())
            & (1 << end_pos) as u16 != 0;
    }

    fn set_end_mask_bit(&mut self, code: usize, start_pos: usize, end_pos: usize) {
        let mut mutex_guard = self.end_mask[code][start_pos].lock().unwrap();
        *mutex_guard |= (1 << end_pos) as u16
    }

    fn get_processing_status(&self, code: usize) -> ProcessingStatus {
        *(self.processing_status[code].lock().unwrap())
    }

    fn set_processing_status(&mut self, code: usize, new_processing_status: ProcessingStatus) {
        let mut mutex_guard =
            self.processing_status[code].lock().unwrap();
        *mutex_guard = new_processing_status
    }

    //Time: O(n!*n / p)
    fn init_reduced_code(&mut self) {
        self.reduced_code = self
            .get_code_range()
            .par_iter_mut()
            .map(|code| {
                let permutation = permutation_label::
                get_permutation_from_lehmer_code(self.size, *code);
                let reduced_permutation =
                    permutation_helper::reduce_permutation(&permutation);
                let reduced_code = permutation_label::
                get_lehmer_code_from_permutation(&reduced_permutation);
                return (reduced_permutation.len(), reduced_code);
            }).collect();
    }

    fn get_code_range(&self) -> Vec<usize> {
        (0..constants::FACTORIALS[self.size] as usize)
            .collect::<Vec<usize>>()
    }

    fn init_distance(&mut self, previous_size_data: &Vec<PermutationsData>) {
        self.distance = self.get_code_range()
            .par_iter_mut()
            .map(|code| {
                if self.reduced_code[*code].0 == self.size {
                    return Mutex::from(99 as u8);
                } else {
                    let reduced_size = self.reduced_code[*code].0;
                    let reduced_code = self.reduced_code[*code].1 as usize;
                    let reduced_dist = previous_size_data[reduced_size]
                        .get_distance(reduced_code);
                    return Mutex::from(reduced_dist);
                }
            }).collect();
        return;
    }

    fn init_end_mask(&mut self) {
        self.end_mask = self.get_code_range()
            .par_iter_mut()
            .map(|code| {
                let mut masks: Vec<Mutex<u16>> = vec![];
                for i in 0..self.size {
                    masks.push(Mutex::from(0));
                }
                return masks;
            }).collect();
    }

    fn update_end_mask_and_distance(&mut self, adj_codes: Vec<(usize, usize, usize)>, new_dist: u8) {
        let update_distance_iter: Vec<&Mutex<u8>> = adj_codes.par_iter()
            .map(|adj_config| &self.distance[adj_config.0])
            .collect();

        let update_end_mask_bit_iter: Vec<(&Mutex<u16>, usize)> = adj_codes.par_iter()
            .map(|adj_config| (&self.end_mask[adj_config.0][adj_config.1], adj_config.2))
            .collect();

        update_distance_iter.par_iter()
            .for_each(|iter| {
                let mut mutex_guard = (iter).lock().unwrap();
                *mutex_guard = new_dist
            });

        update_end_mask_bit_iter.par_iter()
            .for_each(|iter_value_pair| {
                let iter = iter_value_pair.0;
                let bit_to_set = iter_value_pair.1;
                let mut mutex_guard = (iter).lock().unwrap();
                *mutex_guard = (*mutex_guard) | ((1 << bit_to_set) as u16);
            });

//        let update_end_bit_iter = adj_codes.par_iter()
//            .map(|adj_config| self.end_mask[adj_config.0][adj_config.1])
//            .for_each(|adj_config| {
//                let new_code = adj_config.0;
//                let new_i = adj_config.1;
//                let new_j = adj_config.2;
//                self.set_distance(new_code, new_dist);
//                self.set_end_mask_bit(new_code, new_i, new_j);
//            });
    }

    fn process_permutation_paralleled(&mut self, code: usize) {
        if DEBUG_ENABLED {
            println!("Processing code: {}", code);
        }
        fn process_permutation_block_slide(
            permutations_data: &PermutationsData, code: usize, i: usize, j: usize)
            -> Vec<(usize, usize, usize)> {
            if permutations_data.get_end_mask_bit(code, i, j) {
                return vec![];
            }
            let block_size = (j - i) + 1;
            let mut cur_code = code;
            let mut new_permutations_to_process: Vec<(usize, usize, usize)> = vec![];

            for k in j + 1..permutations_data.size {
                let block_shift = k - j;
                let new_i = i + block_shift;
                let new_j = j + block_shift;
                let new_pos = k as i8 - block_size as i8 - 1;
                let delta_for_move = permutations_data.adjacency_calculator
                    .get_delta_for_moving_item(cur_code as u32, k, new_pos);

                cur_code = (cur_code as i32 + delta_for_move) as usize;

                let processing_status = permutations_data.get_processing_status(cur_code);

                if processing_status == PROCESSING
                    || processing_status == PROCESSED {
                    break;
                } else if permutations_data.get_end_mask_bit(cur_code, new_i, new_j) {
                    break;
                } else {
                    new_permutations_to_process.push((cur_code, new_i, new_j));
                }
            }
            new_permutations_to_process
        }

        let adjacent_permutations: Vec<(usize, usize, usize)> =
            get_all_blocks_for_size(self.size)
                .par_iter_mut()
                .map(|pair| process_permutation_block_slide(
                    &self, code as usize, pair.0, pair.1))
                .flatten()
                .collect();

        if DEBUG_ENABLED {
            println!("Processing: {}, adjacent_permutations: {:?}", code, adjacent_permutations);
        }

        //todo: parallelize this!

        let new_dist = self.get_distance(code);

        self.update_end_mask_and_distance(adjacent_permutations, self.get_distance(code) + 1);
    }

    fn set_codes_to_processed_with_distance_less_than(&mut self, dist: u8) {
        self.get_code_range().par_iter_mut()
            .filter(|code| *(self.distance[**code].lock().unwrap()) < dist)
            .for_each(|code| {
                let mut mutex_guard = self.processing_status[*code].lock().unwrap();
                *mutex_guard = PROCESSED;
            });
    }

    fn get_starting_dist(&self) -> u8 {
        let max_reduced_size_adjacent = max(self.size as i8 - 3, 0) as u8;
        let found_min = self.get_code_range()
            .par_iter()
            .filter(|code| self.reduced_code[**code].0 == max_reduced_size_adjacent as usize)
            .map(|code| *(self.distance[*code].lock().unwrap()))
            .min();
        let dist: u8 = match found_min { Some(x) => x, None => 0 };
        if dist == 99 {
            0
        } else {
            dist
        }
    }

    fn process_permutations(&mut self) {
        if DEBUG_ENABLED {
            println!("Processing Permutations of size: {}", self.size);
        }
        //todo: shouldn't this start with n - 3??
        let mut current_dist = self.get_starting_dist();
        println!("Starting_Dist: {}", current_dist);
        let mut max_dist: u8 = self.get_max_dist();
        let now = Instant::now();
        let mut current_time = now.elapsed();

        self.set_codes_to_processed_with_distance_less_than(current_dist);

        while current_dist + 2 < max_dist {
            let prev_time_ms = current_time.as_millis();
            let current_batch: Vec<usize> = self.get_code_range()
                .par_iter_mut()
                .filter(|code| self.get_distance(**code) == current_dist)
                .map(|code| *code as usize)
                .collect();

            for code in &current_batch {
                self.set_processing_status(*code, PROCESSING);
            }

            for code in &current_batch {
                self.process_permutation_paralleled(*code);
            }

            current_time = now.elapsed();
            let cur_time_ms = current_time.as_millis();

            if DEBUG_ENABLED {
                let distances: Vec<u8> = current_batch.par_iter()
                    .map(|code| self.get_distance(*code))
                    .collect();
                println!("\tProcessing distance: {}, Max_dist = {},\
             \n\tcurrent_batch: {:?}\
             \n\tdistances: {:?}", current_dist, max_dist, current_batch, distances);
            }

            current_dist += 1;
            max_dist = self.get_max_dist();
        }
    }
    fn init_processing_status(&mut self) {
        self.processing_status = self.get_code_range()
            .par_iter_mut()
            .map(|code| Mutex::from(UNPROCESSED))
            .collect();
    }

    pub fn init(size: usize, previous_size_data: &Vec<PermutationsData>) -> PermutationsData {
        let mut permutations_data = PermutationsData {
            size,
            distance: vec![],
            adjacency_calculator: AdjacencyCalculator::init(size),
            reduced_code: vec![],
            processing_status: vec![],
            end_mask: vec![],
        };
        permutations_data.init_reduced_code();
        permutations_data.init_distance(&previous_size_data);
        permutations_data.init_end_mask();
        permutations_data.init_processing_status();
        permutations_data.process_permutations();
        return permutations_data;
    }
}


pub fn generate_data_for_size_up_to(max_size: usize) -> Vec<PermutationsData> {
    let mut permutations_data = vec![
        PermutationsData {
            size: 0,
            distance: vec![Mutex::from(0)],
            adjacency_calculator: AdjacencyCalculator::init(0),
            reduced_code: vec![(0, 0)],
            processing_status: vec![],
            end_mask: vec![],
        },
    ];
    for size in 1..max_size {
        if DEBUG_ENABLED {
            println!("Generating for size: {}", size);
        }
        let now = Instant::now();
        let mut cur_size_permutations_data =
            PermutationsData::init(size, &permutations_data);
        /*
        let elapsed_on_init = now.elapsed();
        if DEBUG_ENABLED {
            println!("For size {}, init time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_init.as_millis(),
                     elapsed_on_init.as_secs());
        }
        cur_size_permutations_data.process_all_permutations_time_optimized();
        let elapsed_on_process_pure_permutations = now.elapsed();
        if DEBUG_ENABLED {
            println!("For size {}, process_pure_permutations: time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_process_pure_permutations.as_millis() - elapsed_on_update_init.as_millis(),
                     elapsed_on_process_pure_permutations.as_secs() - elapsed_on_update_init.as_secs());
        }
        */
        permutations_data.push(cur_size_permutations_data);
    }
    return permutations_data;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestCase {
        size: u8,
        permutations_data: PermutationsData,
        expected_pure_permutations: Vec<Vec<u8>>,
    }

    #[test]
    fn test_init_data() {
        let permutations_data = generate_data_for_size_up_to(12);
        for data in permutations_data {
            data.print_summary();
        }
    }

    #[test]
    fn mutex_test() {
        let mut mutex_state: Vec<Vec<Mutex<u16>>> = vec![];
        let size: usize = 5;
        for code in 0..constants::FACTORIALS[size] {
            let mut code_vec: Vec<Mutex<u16>> = vec![];
            for i in 0..size {
                let value = 0 as u16;
                code_vec.push(Mutex::from(value));
            }
            mutex_state.push(code_vec);
        }
        let update: Vec<(usize, usize, usize)> = vec![(0, 0, 0), (0, 1, 1), (1, 1, 1)];
        update.par_iter()
            .for_each(|item| {
                let code = item.0;
                let i = item.1;
                let j = item.2;
                let mut update_code = mutex_state[code][i].lock().unwrap();
                *update_code += j as u16;
            });
    }
}