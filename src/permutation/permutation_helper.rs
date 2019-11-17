const DEBUG_ENABLED: bool = false;

pub fn reduce_permutation(permutation: &Vec<u8>) -> Vec<u8> {
    if DEBUG_ENABLED {
        println!("Reducing Permutation: {:?}", permutation);
    }
    //note: 0-indexed here, 1-indexed in paper
    let size = permutation.len();
    let n = size as u8;
    let mut reduced_permutation: Vec<u8> = vec![];
    let mut offset: Vec<u8> = vec![0; size];

    reduced_permutation.push(permutation[0]);

    let mut position: u8 = 1;
    let mut adjacency_ct_in_block: u8 = 0; // number of adjacencies seen in current block
    let mut reduced_index: u8 = 1;
    let mut adjacency_ct_in_permutation: u8 = 0;

    /*
        Algorithm goes as follows:
        - Identify the blocks of adjacencies
        - Push the first element of each block into reduced_permutation
        - create offsets for second element's value in each block
        - post the calculations, accumulate the offsets
    */

    while position < n {
        while position < n &&
            permutation[position as usize] == permutation[(position - 1) as usize] + 1 {
            adjacency_ct_in_block = adjacency_ct_in_block + 1;
            adjacency_ct_in_permutation = adjacency_ct_in_permutation + 1;
            position = position + 1;
        }
        //block is identified
        if adjacency_ct_in_block > 0 {
            // offset the second symbol of the block = adjacency_ct_in_block
            offset[permutation[(position - adjacency_ct_in_block as u8) as usize] as usize]
                = adjacency_ct_in_block;
        }
        if position < n {
            reduced_permutation.push(permutation[position as usize]);
            // reduced permutations next item is the first item of new block
            position = position + 1;
            adjacency_ct_in_block = 0;
        }
    }
    // accumulate the offsets
    if DEBUG_ENABLED {
        println!("Before Adjustment: {:?}", reduced_permutation);
        println!("Offset Before Prefix Sum: {:?}", offset);
    }
    prefix_sum(&mut offset);
    if DEBUG_ENABLED {
        println!("Offset Post Prefix Sum: {:?}", offset);
    }
    for position in 0..reduced_permutation.len() {
        let current_value = reduced_permutation[position as usize];
        reduced_permutation[position as usize] = current_value - offset[current_value as usize];
    }
    let m = n - adjacency_ct_in_permutation as u8;
    return normalize(reduced_permutation);
}

fn normalize(permutation: Vec<u8>) -> Vec<u8> {
    if DEBUG_ENABLED {
        println!("Normalizing Permutation : {:?}", permutation);
    }
    //todo: implement
    let remove_first_element: bool = permutation[0] == 0;
    let remove_last_element: bool =
        permutation.len() > 1
            && permutation[permutation.len() - 1] == permutation.len() as u8 - 1;

    if DEBUG_ENABLED {
        println!("remove_first_element : {:?}\nremove_last_element : {}",
                 remove_first_element, remove_last_element);
    }

    let mut normalized_permutation: Vec<u8> = vec![];
    if remove_first_element {
        for position in 1..permutation.len() {
            normalized_permutation.push(permutation[position as usize] - 1);
        }
    } else {
        for value in permutation {
            normalized_permutation.push(value);
        }
    }
    if remove_last_element {
        normalized_permutation.remove(normalized_permutation.len() - 1);
    }

    if DEBUG_ENABLED {
        println!("reduced_permutation: {:?}", normalized_permutation);
    }
    return normalized_permutation;
}

fn prefix_sum(offset: &mut Vec<u8>) -> () {
    for i in 1..offset.len() as usize {
        offset[i] = offset[i - 1] + offset[i];
    }
}


#[test]
pub fn test_reduce_permutation() {
    let permutations: Vec<Vec<u8>> = vec![
        vec![0, 1, 4, 3, 2],
        vec![0, 1, 2],
        vec![0, 1, 2, 3],
        vec![0, 2, 1, 3],
        vec![3, 2, 1, 0]
    ];

    let expected_reduced_form: Vec<Vec<u8>> = vec![
        vec![2, 1, 0],
        vec![],
        vec![],
        vec![1, 0],
        vec![3, 2, 1, 0]
    ];

    let mut actual_reduced_form: Vec<Vec<u8>> = vec![];

    for permutation in permutations {
        actual_reduced_form.push(reduce_permutation(&permutation));
    }

    for test_case_number in 0..actual_reduced_form.len() {
        assert_eq!(actual_reduced_form[test_case_number],
                   expected_reduced_form[test_case_number]);
    }
}