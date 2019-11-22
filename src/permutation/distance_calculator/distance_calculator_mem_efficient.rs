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

// Time Complexity: O(n!*n^2 / t)
// Space Complexity: O(n!*n)

// Where t is number of concurrent threads
const DEBUG_ENABLED: bool = false;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ProcessingStatus {
    UNPROCESSED,
    PROCESSING,
    PROCESSED,
}

pub struct PermutationsData {
    size: usize,
    distance: Vec<u8>,
    adjacency_calculator: AdjacencyCalculator,
    reduced_code: Vec<(usize, u32)>,
    processing_status: Vec<ProcessingStatus>,
    end_mask: Vec<Vec<u16>>,
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

fn log_time_taken(prev_duration: &mut Duration, cur_duration: &mut Duration, name: &str) {
    let time_taken_ms = cur_duration.as_millis() - prev_duration.as_millis();
    let time_taken_s = cur_duration.as_secs() - prev_duration.as_secs();
    println!("{} took : {} ms, {} s", name, time_taken_ms, time_taken_s);
}


//todo: space complexity can be reduced by closely packing some memos
impl PermutationsData {
    fn print(&self) {
        println!("\t size: {:?}", self.size);
        println!("\t distance: {:?}", self.distance);
//        println!("\t adjacency_calculator: {:?}", self.adjacency_calculator);
        println!("\t reduced_code: {:?}", self.reduced_code);
        println!("\t end_mask: {:?}", self.end_mask);
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
                    return 99 as u8;
                } else {
                    previous_size_data[self.reduced_code[*code].0 as usize]
                        .distance[self.reduced_code[*code].1 as usize]
                }
            }).collect();
        return;
    }

    fn init_end_mask(&mut self) {
        self.end_mask = self.get_code_range()
            .par_iter_mut()
            .map(|code| {
                let mut masks: Vec<u16> = vec![];
                for i in 0..self.size {
                    masks.push(0);
                }
                return masks;
            }).collect();
    }

    fn process_permutation_block_slide(&mut self, code: usize, i: usize, j: usize) {
        if self.end_mask[code][i] & (1 << j) as u16 == 0 {
            return;
        }
        let block_size = (j - i) + 1;
        let mut cur_code = code;

        for k in j + 1..self.size {
            let block_shift = k - j;
            let new_i = i + block_shift;
            let new_j = j + block_shift;
            let new_pos = k as i8 - block_size as i8 - 1;
            let delta_for_move = self.adjacency_calculator
                .get_delta_for_moving_item(cur_code as u32, k, new_pos);

            cur_code = (cur_code as i32 + delta_for_move) as usize;

            if self.processing_status[cur_code] == PROCESSING
                || self.processing_status[cur_code] == PROCESSED {
                break;
            } else if self.end_mask[cur_code][new_i] & (1 << new_j) as u16 == 1 {
                break;
            } else {
                self.end_mask[cur_code][new_i] |= (1 << new_j) as u16;
            }
        }
    }

    fn process_permutation_paralleled(&mut self, code: usize) {
        get_all_blocks_for_size(self.size)
            .par_iter_mut()
            .for_each(|pair| self.process_permutation_block_slide(code as usize, pair.0, pair.1));
    }

    fn process_permutations(&mut self) {
        let mut current_dist = max(0, self.size as i8 - 3) as u8;
        let mut max_dist: u8 = *(self.distance.par_iter_mut().max().unwrap());
        while current_dist + 2 < max_dist {
            let current_batch: Vec<usize> = self.get_code_range()
                .par_iter_mut()
                .filter(|code| self.distance[**code] != current_dist)
                .map(|code| *code as usize)
                .collect();

            for code in &current_batch {
                self.process_permutation_paralleled(*code);
            }

            current_dist += 1;
            max_dist = *(self.distance.par_iter_mut().max().unwrap());
        }
    }
    fn init_processing_status(&mut self) {
        self.processing_status = self.get_code_range()
            .par_iter_mut()
            .map(|code| UNPROCESSED)
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
        permutations_data.process_permutations();
        permutations_data.init_processing_status();
        return permutations_data;
    }
}


pub fn generate_data_for_size_up_to(max_size: usize) -> Vec<PermutationsData> {
    let mut permutations_data = vec![
        PermutationsData {
            size: 0,
            distance: vec![0],
            adjacency_calculator: AdjacencyCalculator::init(0),
            reduced_code: vec![(0, 0)],
            processing_status: vec![],
            end_mask: vec![],
        },
    ];
    for size in 1..max_size {
        if !DEBUG_ENABLED {
            println!("Generating for size: {}", size);
        }
        let now = Instant::now();
        let mut cur_size_permutations_data =
            PermutationsData::init(size, &permutations_data);
        /*
        let elapsed_on_init = now.elapsed();
        if !DEBUG_ENABLED {
            println!("For size {}, init time taken (ms) = {:?}, (s) = {:?}",
                     size,
                     elapsed_on_init.as_millis(),
                     elapsed_on_init.as_secs());
        }
        cur_size_permutations_data.process_all_permutations_time_optimized();
        let elapsed_on_process_pure_permutations = now.elapsed();
        if !DEBUG_ENABLED {
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

    struct TestCase {
        size: u8,
        permutations_data: PermutationsData,
        expected_pure_permutations: Vec<Vec<u8>>,
    }

    #[test]
    fn test_init_data() {
        let permutations_data = generate_data_for_size_up_to(5);
        for data in permutations_data {
//            data.print();
        }
    }
}