use std::fs::File;
use std::io::{self, Read, Write};
use tempfile::tempdir;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() {
    // Assuming the script is run from the directory containing SaveData.zip
    let zip_path = "SaveData.zip";
    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Step 1: Extract the zip file
    println!("Opening zip file...");
    let zip_file = File::open(zip_path).expect("Failed to open zip file");
    let mut archive = ZipArchive::new(zip_file).expect("Failed to read zip archive");

    let temp_extract_path = temp_dir.path().join("GameStateSaveData.json");
    {
        println!("Extracting GameStateSaveData.json...");
        let mut file = archive.by_name("GameStateSaveData.json").expect("Failed to find GameStateSaveData.json in archive");
        let mut extracted_file = File::create(&temp_extract_path).expect("Failed to create temp file for extraction");
        std::io::copy(&mut file, &mut extracted_file).expect("Failed to copy contents of GameStateSaveData.json");
    }

    // Step 2: Modify the extracted file
    println!("Modifying GameStateSaveData.json...");
    let mut file_contents = String::new();
    {
        let mut extracted_file = File::open(&temp_extract_path).expect("Failed to open extracted file");
        extracted_file.read_to_string(&mut file_contents).expect("Failed to read extracted file");
    }

    println!("Removing IsRobbyDead:true value...");
    file_contents = file_contents.replace("\\\"IsRobbyDead\\\":true,", "");

    {
        let mut modified_file = File::create(&temp_extract_path).expect("Failed to create temp file for modification");
        modified_file.write_all(file_contents.as_bytes()).expect("Failed to write modified contents");
    }

    // Step 3: Repack the zip file with the modified json
    println!("Repacking SaveData.zip...");
    let temp_zip_path = temp_dir.path().join("SaveData_modified.zip");
    {
        let temp_zip_file = File::create(&temp_zip_path).expect("Failed to create temp zip file");
        let mut zip_writer = ZipWriter::new(temp_zip_file);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to access file in zip archive");
            let file_name = file.name().to_string();
            if file_name == "GameStateSaveData.json" {
                zip_writer.start_file(file_name, FileOptions::default()).expect("Failed to start file in zip");
                let mut modified_file = File::open(&temp_extract_path).expect("Failed to open modified file");
                std::io::copy(&mut modified_file, &mut zip_writer).expect("Failed to copy modified contents to zip");
            } else {
                zip_writer.start_file(file_name, FileOptions::default()).expect("Failed to start file in zip");
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
