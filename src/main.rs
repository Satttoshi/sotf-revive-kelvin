use std::fs::File;
use std::io::{self, Read, Write};
use tempfile::tempdir;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use zip::ZipWriter;

fn extract_file_from_zip(archive: &mut ZipArchive<File>, file_name: &str, output_path: &std::path::Path) {
    println!("Extracting {}...", file_name);
    let mut file = archive.by_name(file_name).expect("Failed to find file in archive");
    let mut extracted_file = File::create(output_path).expect("Failed to create temp file for extraction");
    std::io::copy(&mut file, &mut extracted_file).expect("Failed to copy contents of file");
}

fn read_file_to_string(file_path: &std::path::Path) -> String {
    let mut file_contents = String::new();
    let mut file = File::open(file_path).expect("Failed to open file");
    file.read_to_string(&mut file_contents).expect("Failed to read file");
    file_contents
}

fn write_string_to_file(file_path: &std::path::Path, contents: &str) {
    let mut file = File::create(file_path).expect("Failed to create file");
    file.write_all(contents.as_bytes()).expect("Failed to write contents to file");
}

fn find_second_occurrence(s: &str, pattern: &str) -> Option<usize> {
    let first_occurrence = s.find(pattern)?;
    s[first_occurrence + pattern.len()..].find(pattern).map(|second| first_occurrence + pattern.len() + second)
}

fn main() {
    // Assuming the script is run from the directory containing SaveData.zip
    let zip_path = "SaveData.zip";
    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Step 1: Extract the zip file
    println!("Opening zip file...");
    let zip_file = File::open(zip_path).expect("Failed to open zip file");
    let mut archive = ZipArchive::new(zip_file).expect("Failed to read zip archive");

    // Paths to temporary files
    let temp_game_state_path = temp_dir.path().join("GameStateSaveData.json");
    let temp_save_data_path = temp_dir.path().join("SaveData.json");


    // Extract GameStateSaveData.json
    extract_file_from_zip(&mut archive, "GameStateSaveData.json", &temp_game_state_path);

    // Step 2: Modify GameStateSaveData.json
    println!("Modifying GameStateSaveData.json...");
    let mut game_state_contents = read_file_to_string(&temp_game_state_path);
    game_state_contents = game_state_contents.replace("\\\"IsRobbyDead\\\":true,", "");
    write_string_to_file(&temp_game_state_path, &game_state_contents);


    // Extract SaveData.json
    extract_file_from_zip(&mut archive, "SaveData.json", &temp_save_data_path);

    // Modify SaveData.json
    println!("Modifying SaveData.json...");
    let mut save_data_contents = read_file_to_string(&temp_save_data_path);

    // Change "State" value from 6 to 2 for the first "TypeId":9 object
    if let Some(pos) = save_data_contents.find("\\\"TypeId\\\":9,\\\"Position\\\":") {
        if let Some(state_pos) = save_data_contents[pos..].find("\\\"State\\\":6") {
            let state_index = pos + state_pos;
            save_data_contents.replace_range(state_index..state_index + "\\\"State\\\":6".len(), "\\\"State\\\":2");
        }
    }

    // Add "Health":100.0 to the "Stats" object of the first "TypeId":9 object
    if let Some(pos) = save_data_contents.find("\\\"TypeId\\\":9,\\\"Position\\\":") {
        if let Some(stats_pos) = save_data_contents[pos..].find("\\\"Stats\\\":") {
            let stats_index = pos + stats_pos;
            if let Some(end_brace_pos) = save_data_contents[stats_index..].find('}') {
                let insert_index = stats_index + end_brace_pos;
                save_data_contents.insert_str(insert_index, ",\\\"Health\\\":100.0");
            }
        }
    }

    // Modify the second occurrence of "TypeId":9
    if let Some(second_pos) = find_second_occurrence(&save_data_contents, "\\\"TypeId\\\":9") {
        if let Some(player_killed_pos) = save_data_contents[second_pos..].find("\\\"PlayerKilled\\\":1") {
            let player_killed_index = second_pos + player_killed_pos;
            if let Some(end_comma_pos) = save_data_contents[player_killed_index..].find(',') {
                let end_index = player_killed_index + end_comma_pos + 1;
                save_data_contents.replace_range(player_killed_index..end_index, "");
            }
        }

        if let Some(killed_on_day_pos) = save_data_contents[second_pos..].find("\\\"KilledOnDay\\\":") {
            let killed_on_day_index = second_pos + killed_on_day_pos;
            if let Some(start_brace_pos) = save_data_contents[killed_on_day_index..].find('{') {
                if let Some(end_brace_pos) = save_data_contents[killed_on_day_index..].find('}') {
                    let start_index = killed_on_day_index + start_brace_pos;
                    let end_index = killed_on_day_index + end_brace_pos + 1;
                    save_data_contents.replace_range(start_index + 1..end_index - 1, "");
                }
            }
        }
    }

    write_string_to_file(&temp_save_data_path, &save_data_contents);

    // Step 3: Repack the zip file with the modified json files
    println!("Repacking SaveData.zip...");
    let temp_zip_path = temp_dir.path().join("SaveData_modified.zip");
    {
        let temp_zip_file = File::create(&temp_zip_path).expect("Failed to create temp zip file");
        let mut zip_writer = ZipWriter::new(temp_zip_file);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to access file in zip archive");
            let file_name = file.name().to_string();
            zip_writer.start_file(file_name.clone(), FileOptions::default()).expect("Failed to start file in zip");
            if file_name == "GameStateSaveData.json" {
                let mut modified_file = File::open(&temp_game_state_path).expect("Failed to open modified file");
                std::io::copy(&mut modified_file, &mut zip_writer).expect("Failed to copy modified contents to zip");
            } else if file_name == "SaveData.json" {
                let mut modified_file = File::open(&temp_save_data_path).expect("Failed to open modified file");
                std::io::copy(&mut modified_file, &mut zip_writer).expect("Failed to copy modified contents to zip");
            } else {
                std::io::copy(&mut file, &mut zip_writer).expect("Failed to copy original contents to zip");
            }
        }

        zip_writer.finish().expect("Failed to finish zip file");
    }

    // Replace the original zip with the modified one
    println!("Replacing the original zip file...");
    std::fs::copy(&temp_zip_path, zip_path).expect("Failed to replace original zip file with modified one");

    println!("Process completed. Press Enter to exit.");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
