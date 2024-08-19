use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use colored::*;
use serde::{Deserialize, Serialize};
use slint::Color;

slint::include_modules!();

const VERSION: &str = "0.0.1";

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    editor_font_family: String,
    editor_font_size: String,
    editor_font_weight: String,
    editor_font_color: String,
    editor_line_num_color: String,
    background_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor_font_family: "JetBrains Mono".to_string(),
            editor_font_size: "18px".to_string(),
            editor_font_weight: "300".to_string(),
            editor_font_color: "#e9e9e9".to_string(),
            editor_line_num_color: "#525252".to_string(),
            background_color: "#1e1e1ecd".to_string(),
        }
    }
}

fn load_config(path: &str) -> Result<Config, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader).unwrap_or_default();
    Ok(config)
}

fn hex_to_color(hex: &str) -> Result<Color, &'static str> {
    // Adjust the color parsing logic based on the expected format
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[3..5], 16).map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[5..7], 16).map_err(|_| "Invalid blue component")?;
        Ok(Color::from_rgb_u8(r, g, b))
    } else if hex.len() == 9 && hex.starts_with('#') {
        let a = u8::from_str_radix(&hex[1..3], 16).map_err(|_| "Invalid alpha component")?;
        let r = u8::from_str_radix(&hex[3..5], 16).map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[5..7], 16).map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[7..9], 16).map_err(|_| "Invalid blue component")?;
        Ok(Color::from_argb_u8(a, r, g, b))
    } else {
        Err("Invalid color format")
    }
}

fn main() {
    let config = match load_config("./config/config.json") {
        Ok(config) => {
            println!("Config loaded: {:?}", config);
            config
        }
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

    let seth = Seth::new().unwrap();

    seth.set_version(seth_version(VERSION).into());

    // Configuring the editor based on loaded configuration
    configure_editor(&seth, &config);

    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // Handle command-line arguments
    if !handle_command_line_args(&seth, action, &args) {
        return;
    }

    seth.run().unwrap();

    // Automatically save file on exit
    if let Some(file_path) = args.get(2) {
        save_file(Path::new(file_path), &seth.get_file_contents());
    }
}

fn configure_editor(seth: &Seth, config: &Config) {
    seth.set_editor_font_family(config.editor_font_family.clone().into());

    if let Ok(size) = config.editor_font_size.parse::<f32>() {
        seth.set_editor_font_size(size);
    }

    if let Ok(weight) = config.editor_font_weight.parse::<i32>() {
        seth.set_editor_font_weight(weight);
    }

    if let Ok(color) = hex_to_color(&config.editor_font_color) {
        seth.set_editor_font_color(color);
    }

    if let Ok(color) = hex_to_color(&config.editor_line_num_color) {
        seth.set_editor_line_num_color(color);
    }

    if let Ok(color) = hex_to_color(&config.background_color) {
        seth.set_background_color(color);
    }
}

fn handle_command_line_args(seth: &Seth, action: &str, args: &[String]) -> bool {
    match action {
        "--help" => {
            print_help();
            false
        }
        "--version" => {
            println!("Seth version {}", VERSION);
            false
        }
        "new" => {
            handle_new_file(seth, args);
            true
        }
        "edit" => {
            handle_edit_file(seth, args);
            true
        }
        "config" => {
            print_config();
            false
        }
        _ => {
            eprintln!("Error: Unknown action '{}'", action);
            print_help();
            false
        }
    }
}

fn print_help() {
    println!("Usage: 'Seth' <action> <optional identifier/path> <optional initial_content>");
    println!("Actions:");
    println!("  --help        Prints this help message");
    println!("  --version     Prints the Seth version");
    println!("  new           Creates a new file");
    println!("  edit          Edits an existing file");
    println!("  config        Prints the current configuration");
}

fn handle_new_file(seth: &Seth, args: &[String]) {
    let default = String::from("text.txt");
    let empty = String::from("");
    let initial_content = args.get(3).unwrap_or(&empty);
    let identifier = args.get(2).unwrap_or(&default);

    println!("Creating new file: {}", identifier);
    if !initial_content.is_empty() {
        println!("With initial content: '{}'", initial_content);
    }

    let raw_path = match Path::new(identifier).canonicalize() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Error canonicalizing {identifier}\n\terr -> {err}");
            return;
        }
    };

    let cleaned_path = to_cleaned_path(&raw_path.to_str().to_owned().unwrap_or("").to_string());

    // Write the initial content to the file
    if let Err(err) = fs::write(&raw_path, initial_content) {
        eprintln!("Error writing initial content to {identifier}\n\terr -> {err}");
        return;
    }

    seth.set_file_name(get_file_name(Path::new(identifier)).into());
    seth.set_file_path(slint::SharedString::from(cleaned_path));
    seth.set_file_contents(initial_content.into());
}

fn handle_edit_file(seth: &Seth, args: &[String]) {
    let empty = String::from("");
    let identifier = args.get(2).unwrap_or(&empty);
    if identifier.is_empty() {
        eprintln!("Error: No file path provided for edit action");
    } else if identifier == "config" {
        println!("Opening config file: ./config/config.json");
        // Add logic to open "./config/config.json"
    } else {
        println!("Editing file: {}", identifier);
        // Example logic to read file contents and set in Seth editor
        let raw_path = match Path::new(identifier).canonicalize() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("Error canonicalizing {identifier}\n\terr -> {err}");
                return;
            }
        };

        let cleaned_path = to_cleaned_path(&raw_path.to_str().to_owned().unwrap_or("").to_string());

        if let Ok(contents) = fs::read_to_string(&raw_path) {
            seth.set_file_name(get_file_name(Path::new(identifier)).into());
            seth.set_file_path(slint::SharedString::from(cleaned_path.replace("\\", "/")));
            seth.set_file_contents(contents.clone().replace("\r\n", "\n").into());

            println!("{}", contents.clone());

            // Automatically generate and set line numeration
            let num_of_lines = contents.lines().count();
            let line_numeration = generate_line_numeration(&num_of_lines);
            seth.set_line_numeration(line_numeration.into());
        } else {
            eprintln!("Error: Failed to read the file '{}'", identifier);
        }
    }
}

fn get_file_name(file_path: &Path) -> String {
    file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown file name")
        .to_string()
}

fn to_cleaned_path(raw_abs_path: &String) -> String {
    if raw_abs_path.starts_with(r"\\?\") {
        // Return raw_abs_path minus the 4 first characters
        raw_abs_path[4..].to_owned()
    } else {
        raw_abs_path.to_owned()
    }
}

fn generate_line_numeration(num_of_lines: &usize) -> String {
    let mut line_numeration = String::new();
    for line_number in 1..=*num_of_lines {
        line_numeration.push_str(&line_number.to_string());
        line_numeration.push('\n');
    }
    line_numeration
}

fn print_config() {
    println!("Current configuration:");
    // Add logic to print the current configuration
}

fn save_file(file_path: &Path, content: &str) {
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

fn seth_version(version: &str) -> String {
    format!("Seth v{}", version)
}
