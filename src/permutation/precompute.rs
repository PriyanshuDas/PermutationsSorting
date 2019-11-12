use super::permutation_helper;
use super::permutation_label;

const DEBUG_ENABLED: bool = false;


//todo: fix moved values errors
pub fn pre_compute(pure_perm_labels: Vec<u32>, size: usize) -> Vec<Vec<Vec<u32>>> {
    //dimensions: [labels][size][size]
    let total_permutation_labels = permutation_label::sum_factorial(size as u32);
    let mut grid_3d = initialize_3d_grid(
        total_permutation_labels as usize,
        size,
        size);

    for label in pure_perm_labels {
        let permutation = permutation_label::get_permutation_from_label(label);
        for i in 0..size {
            for j in i..size {
                let mut new_permutation: Vec<u8> =
                    permutation_move_item(&permutation, j, i);

                let new_label = permutation_label::get_label_for_permutation(&new_permutation);
                if DEBUG_ENABLED {
                    println!("Old Permutation: {:?}\t moving ({} to before {})\n\
                New Permutation: {:?}, New label: {}", &permutation, permutation[j], permutation[i],
                             &new_permutation, new_label)
                };
            }
        }
    }

    if DEBUG_ENABLED {
        print_3d_grid(&mut grid_3d)
    }
    return grid_3d;
}

fn permutation_move_item(permutation: &Vec<u8>, item_pos: usize, move_to_before: usize) -> Vec<u8> {
    let mut new_permutation: Vec<u8> = vec![];
    for pos in 0..move_to_before {
        new_permutation.push(permutation[pos]);
    }
    new_permutation.push(permutation[item_pos]);
    for pos in move_to_before..item_pos {
        new_permutation.push(permutation[pos]);
    }
    for pos in item_pos+1..permutation.len() {
        new_permutation.push(permutation[pos]);
    }
    new_permutation
}

fn initialize_3d_grid(d1: usize, d2: usize, d3: usize) -> Vec<Vec<Vec<u32>>> {
    let mut grid_3d =
        vec![vec![vec![0 as u32; d3]; d2]; d1 as usize];
    return grid_3d;
}

fn print_3d_grid(grid_3d: &mut Vec<Vec<Vec<u32>>>) -> () {
    for row in grid_3d {
        for col in row {
            print!(" |{:?}| ", col);
        }
        println!();
    }
}


#[test]
fn test_pre_compute() {
    let labels = vec![4, 5, 6, 7, 8, 9];
    pre_compute(labels, 3);
}