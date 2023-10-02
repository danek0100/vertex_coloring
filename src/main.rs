use std::collections::HashMap;
use rand::seq::SliceRandom;
use std::fs::File;
use csv::Writer;
use std::fs;
use std::time::Instant;
use bit_set::BitSet;
use rayon::prelude::*;
use std::sync::Mutex;
use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelRefIterator;

fn get_adjacency_matrix(amount_ver: usize, data: &[(usize, usize)]) -> Vec<BitSet> {
    let mut adjacency_matrix = vec![BitSet::with_capacity(amount_ver); amount_ver];

    for &(u, v) in data.iter() {
        adjacency_matrix[u - 1].insert(v - 1);
        adjacency_matrix[v - 1].insert(u - 1);
    }

    adjacency_matrix
}

fn has_neighbours(vert: usize, color_class: &[i32], adjacency_matrix: &[BitSet]) -> bool {
    color_class.iter().any(|&vert_| adjacency_matrix[vert].contains(vert_ as usize))
}

fn get_degree_vert(adjacency_matrix: &[BitSet]) -> HashMap<usize, usize> {
    let mut degree_vertices = HashMap::new();

    for (index, row) in adjacency_matrix.iter().enumerate() {
        degree_vertices.insert(index, row.len());
    }

    degree_vertices
}

fn solver(sorted_keys: &[usize], adjacency_matrix: &[BitSet]) -> (i32, Vec<Vec<i32>>) {
    let mut colour_classes = vec![Vec::new()];
    let mut max_colour = 1;

    for &vertex in sorted_keys {
        let mut current_colour = 1;

        while current_colour <= colour_classes.len() &&
            has_neighbours(vertex, &colour_classes[current_colour - 1], adjacency_matrix) {

            current_colour += 1;
            if current_colour > colour_classes.len() {
                colour_classes.push(Vec::new());
            }
        }

        max_colour = max_colour.max(current_colour);
        colour_classes[current_colour - 1].push(vertex as i32);
    }

    (max_colour as i32, colour_classes)
}

fn get_random(degree_vertices: &HashMap<usize, usize>) -> Vec<usize> {
    let mut grouped_by_degree: Vec<Vec<usize>> = Vec::new();
    let mut max_degree = 0;

    for (&vertex, &degree) in degree_vertices.iter() {
        let degree = degree as usize;
        max_degree = max_degree.max(degree);
        if grouped_by_degree.len() <= degree {
            grouped_by_degree.resize_with(degree + 1, Vec::new);
        }
        grouped_by_degree[degree].push(vertex);
    }

    let mut rng = rand::thread_rng();
    for group in &mut grouped_by_degree {
        group.shuffle(&mut rng);
    }

    let mut ready_list: Vec<usize> = grouped_by_degree.into_iter().flatten().collect();
    ready_list.reverse();

    ready_list
}

fn testing_solution(color_classes: &[Vec<i32>], adjacency_matrix: &[BitSet]) -> &'static str {
    for cluster in color_classes {
        for (n, &vert) in cluster.iter().enumerate() {
            for &vert_2 in &cluster[n + 1..] {
                if adjacency_matrix[vert as usize].contains(vert_2 as usize) {
                    return "TEST FAIL";
                }
            }
        }
    }
    "TEST PASS"
}

struct DataEntry {
    filename: String,
    amount_colors: i32,
    time: f64,
    groups: Vec<i32>,
    test: String,
    optimal_solution: i32,
    solved: bool,
}

fn save_result(result: &Vec<DataEntry>, name_algorithm: &str) {
    let path = format!("{}", name_algorithm);
    let file = File::create(&path).expect("Unable to create file");
    let mut wtr = Writer::from_writer(file);

    wtr.write_record(&["FILENAME", "AMOUNT_COLORS", "TIME", "GROUPS", "TEST", "OPTIMAL_SOLUTION", "SOLVED"])
        .expect("Failed to write headers");

    for entry in result {
        wtr.write_record(&[
            &entry.filename,
            entry.amount_colors.to_string().as_str(),
            entry.time.to_string().as_str(),
            &format!("{:?}", entry.groups),
            &entry.test,
            entry.optimal_solution.to_string().as_str(),
            entry.solved.to_string().as_str(),
        ]).expect("Failed to write record");
    }

    wtr.flush().expect("Failed to write to file");
}

fn create_optimal_solutions() -> HashMap<&'static str, i32>
{
    let mut map = HashMap::new();
    map.insert("myciel3.col.txt", 4);
    map.insert("inithx.i.1.col", 54);
    map.insert("queen11_11.col", 16);
    map.insert("mulsol.i.1.col", 49);
    map.insert("school1.col.txt", 15);
    map.insert("le450_5a.col", 11);
    map.insert("huck.col", 11);
    map.insert("le450_15b.col", 18);
    map.insert("anna.col", 11);
    map.insert("miles1000.col", 42);
    map.insert("myciel7.col.txt", 8);
    map.insert("latin_square_10.col.txt",97);
    map.insert("jean.col.txt", 10);
    map.insert("school1_nsh.col.txt", 21);
    map
}

fn get_testing_data(path: &str) -> (HashMap<String, Vec<(i32, i32)>>, HashMap<String, (i32, i32)>) {
    let mut testing_data: HashMap<String, Vec<(i32, i32)>> = HashMap::new();
    let mut testing_data_amount_info: HashMap<String, (i32, i32)> = HashMap::new();

    for entry in fs::read_dir(path).expect("Failed to read directory") {
        if let Ok(file) = entry {
            let file_name = file.file_name().into_string().expect("Failed to convert OsString to String");
            let file_path = file.path();
            let contents = fs::read_to_string(file_path).expect("Failed to read file");

            let mut data = Vec::new();
            for line in contents.lines() {
                if line.starts_with("p ") {
                    let parts: Vec<&str> = line[6..].split_whitespace().collect();
                    if parts.len() == 2 {
                        let amount_ver: i32 = parts[0].parse().expect("Failed to parse integer");
                        let amount_edge: i32 = parts[1].parse().expect("Failed to parse integer");
                        testing_data_amount_info.insert(file_name.clone(), (amount_ver, amount_edge));
                    }
                } else if line.starts_with("e ") {
                    let parts: Vec<&str> = line[2..].split_whitespace().collect();
                    if parts.len() == 2 {
                        let ver_1: i32 = parts[0].parse().expect("Failed to parse integer");
                        let ver_2: i32 = parts[1].parse().expect("Failed to parse integer");
                        data.push((ver_1, ver_2));
                    }
                }
            }
            testing_data.insert(file_name, data);
        }
    }

    (testing_data, testing_data_amount_info)
}

fn greedy_with_vertex_degree_sort_and_randomize(
    testing_data: &HashMap<String, Vec<(i32, i32)>>,
    testing_data_amount_info: &HashMap<String, (i32, i32)>,
) -> HashMap<String, (i32, f64, Vec<Vec<i32>>, &'static str)> {
    let total_info: Mutex<HashMap<String, (i32, f64, Vec<Vec<i32>>, &'static str)>> = Mutex::new(HashMap::new());


    let keys: Vec<&String> = testing_data.keys().collect();

    keys.par_iter().for_each(|file| {

        let data = match testing_data.get(*file) {
            Some(data) => data,
            None => return, // Skip to the next iteration if file is not found
        };

        let data_info = match testing_data_amount_info.get(*file) {
            Some(info) => info,
            None => return, // Skip to the next iteration if file is not found
        };


        let converted_data: Vec<(usize, usize)> = data
            .iter()
            .map(|&(u, v)| (u as usize, v as usize))
            .collect();

        let mut best_colour = i32::MAX;

        for _ in 0..5000 {
            let adjacency_matrix = get_adjacency_matrix(data_info.0 as usize, &converted_data);
            let degree_vertices = get_degree_vert(&adjacency_matrix);
            let ready_list = get_random(&degree_vertices);

            let time_start = Instant::now();
            let (max_colour, color_classes) = solver(&ready_list, &adjacency_matrix);
            let elapsed_secs = time_start.elapsed().as_secs_f64();

            let test = testing_solution(&color_classes, &adjacency_matrix);
            if max_colour < best_colour {
                best_colour = max_colour;
                let mut total_info = total_info.lock().unwrap();
                total_info.insert(file.to_string(), (max_colour, elapsed_secs, color_classes, test));
            }
        }
    });

    // Extract the HashMap from the Mutex
    total_info.into_inner().unwrap()
}


const PATH: &str = "./input_files";

fn main() {
    let (testing_data, testing_data_amount_info) = get_testing_data(&PATH);
    let result_to_save = greedy_with_vertex_degree_sort_and_randomize(&testing_data, &testing_data_amount_info);

    let mut vec_result = Vec::new();
    for (file, &(amount_colors, time, ref classes, test)) in &result_to_save {
        let groups: Vec<i32> = classes.iter().flatten().cloned().collect();
        let optimal_solutions = create_optimal_solutions();
        let optimal_solution = optimal_solutions.get(file.as_str()).unwrap_or(&i32::MAX);
        vec_result.push(DataEntry {
            filename: file.to_string(),
            amount_colors,
            time,
            groups,
            test: test.to_string(),
            optimal_solution: *optimal_solution,
            solved: amount_colors <= *optimal_solution,
        });
    }

    save_result(&vec_result, "greedy_with_vertex_degree_sort_and_randomize.csv");
}
