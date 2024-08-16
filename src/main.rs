use std::fs;
use std::env;
use std::path::Path;
use colored::*;
use std::io::Write;
use std::fs::File;
use std::process::exit;

slint::include_modules!();

fn main() {
    let version: &str = "0.0.1";
    let args: Vec<String> = env::args().collect();

    let file_path = if let Some(path) = args.get(1) {
        match fs::canonicalize(Path::new(path)) {
            Ok(abs_path) => abs_path,
            Err(err) => {
                eprintln!("Failed to resolve absolute path: {}", err);
                return;
            }
        }
    } else {
        eprintln!("{}", "No file provided.".yellow().bold());
        return;
    };

    let file_name = get_file_name(&file_path);

    let file_contents = match fs::read_to_string(&file_path) {
        Ok(contents) => {
            println!("{}{}: Opening '{}'", "Eden v".magenta(), version.magenta(), &file_name.white());
            contents
        },
        Err(err) => {
            eprintln!("Failed to read the file: {}", err);
            String::from("")
        }
    };

    let eden = Eden::new().unwrap();
    eden.set_version(eden_version(&version).into());
    eden.set_file_path(to_cleaned_path(&file_path.to_string_lossy().into_owned()).into());
    eden.set_file_name(file_name.into());
    eden.set_file_contents(file_contents.into());
    
    let weak_eden_handle = eden.as_weak();
    let mut _file_handle = match File::open(&file_path) {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("Failed to open file: {}", err);
            exit(0);
        }
    };

    eden.on_eventing(move || {
        let weak_eden = weak_eden_handle.unwrap();
        weak_eden.set_file_contents("Hello!".into());
        // file_handle.write_all(eden.get_file_contents().as_bytes()).unwrap();
    });

    eden.run().unwrap();
}

fn eden_version(version: &str) -> String {
    let app_name: String = String::from("Eden v");
    app_name + version
}

fn get_file_name(file_path: &Path) -> String {
    file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown file name")
        .to_string()
}

fn to_cleaned_path(raw_abs_path: &String) -> &str {
    if raw_abs_path.starts_with(r"\\?\") {
        // Return raw_abs_path minus the 4 first characters
        &raw_abs_path[4..]
    } else {
        raw_abs_path
    }
}

fn save_file(file_path: &Path, content: &str) {
    match File::open(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(content.as_bytes()) {
                eprintln!("Failed to write to file: {}", err);
            } else {
                println!("{}", "File saved successfully.".green().bold());
            }
        }
        Err(err) => {
            eprintln!("Failed to create file: {}", err);
        }
    }
}

fn save_as_file(file_path: &Path, content: &str) {
    match File::create(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(content.as_bytes()) {
                eprintln!("Failed to write to file: {}", err);
            } else {
                println!("{}", "File saved successfully.".green().bold());
            }
        }
        Err(err) => {
            eprintln!("Failed to create file: {}", err);
        }
    }
}
