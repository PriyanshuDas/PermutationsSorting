//use std::time::{Instant};
#[macro_use] extern crate text_io;
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::Ordering::AcqRel;
use rand;
use rand::Rng;
use std::cmp::min;

use rayon::prelude::*;

mod permutation;
//use permutation::permutations_generator;
//

const RANGE: usize = 50000000;
fn main() {
    println!("Enter the size of permutations for which to generate: ");
    let n = read!();
    let data =
        permutation::distance_calculator::generate_data_for_size_up_to(n);
    for permutation_size_data in data {
        permutation_size_data.print_summary();
    }
}
#[test]
fn concurrency_test_1() {
    /*
    let handle = thread::spawn(|| {
        "Hello from a thread!"
    });
    println!("{}", handle.join().unwrap());

     below will give error
    let mut x = 1;
    thread::spawn(|| {
        x+= 1;
        println!("x is {}", x);
    }).join();
    println!("x is {}", x);

     will give error
    let mut data = Rc::new(vec![1, 2, 3]);
    for i in 0..3 {
        let data_ref = data.clone();

        thread::spawn(move || {
            data_ref[0] += i;
        });
    }

    thread::sleep(Duration::from_millis(50));

     this won't work either!
    let mut data = Arc::new(vec![1, 2, 3]);

    for i in 0..3 {
        let data = data.clone();
        thread::spawn(move || {
            data[0] += i;
        });
    }
    thread::sleep(Duration::from_millis(50));
    */
    let data = Arc::new(Mutex::new(vec![0;100000]));
    let mut handles = vec![];

    for i in 0..100000 {
        let data = data.clone();
        let handle = thread::spawn(move || {
            let mut data = data.lock().unwrap();
            data[i] += i;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join();
    }

    println!("{:?}", data.lock().unwrap());
}


#[test]
fn concurrency_test_2() {
    let data = Arc::new(Mutex::new(0));
    let (tx, rx) = mpsc::channel();

    for _ in 0..10 {
        let (data, tx) = (data.clone(), tx.clone());

        thread::spawn(move || {
            let mut data = data.lock().unwrap();
            *data += 1;

            tx.send(()).unwrap();
        });
    }

    for _ in 0..10 {
        rx.recv().unwrap();
    }
}

#[test]
fn concurrency_test_double() {
    let (tx, rx) = mpsc::channel();
    let thread_range = 8;

    let batch_ct = RANGE /thread_range + 1;
    let mut current_processing = 0;
    let mut output = vec![];

    while current_processing < RANGE {
        let count_threads_to_spawn = min(thread_range, RANGE - current_processing);
        for i in 0..count_threads_to_spawn {
            let tx = tx.clone();

            if current_processing + i < RANGE {
                thread::spawn(move || {
                    let answer = 2*(current_processing + i);
                    let mut rng = rand::thread_rng();
                    let sleep_time:u8 = rng.gen();
//                    thread::sleep(Duration::from_millis(sleep_time as u64));
                    tx.send((current_processing + i, answer)).unwrap();
                });
            }
        }

        for _ in 0..count_threads_to_spawn {
            let value = rx.recv().unwrap();
            current_processing+= 1;
            output.push(value.1);
        }
    }

//    println!("Output: {:?}", output);
}


#[test]
fn concurrency_test_double_rayon() {
    let mut output = vec![0; RANGE];

    output.par_iter_mut().for_each(|p| *p = 1);

    let mut sum = 0;
    for i in output {
        sum += i;
    }
    println!("sum: {}", sum);
}

#[test]
fn non_concurrent_test_double() {
    let mut output = vec![0; RANGE];

    for i in 0..RANGE {
        output[i] = 1;
    }

    let mut sum = 0;
    for i in output {
        sum += i;
    }
    println!("sum: {}", sum);
//    println!("Output: {:?}", output);
}

#[test]
fn non_concurrent_test() {
        let mut data = vec![0;100000];

        for i in 0..100000 {
            data[i] += i;
        }

        println!("{:?}", data);
}