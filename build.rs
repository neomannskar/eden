fn main() {
  let config = slint_build::CompilerConfiguration::new()
  .with_style("fluent-dark".into());
  slint_build::compile_with_config("./ui/appwindow.slint", config).unwrap();
}


/*

let config = match load_config("./config/config.json") {
        Ok(config) => {
            println!("Config loaded: {:?}", config);
            config
        },
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return;
        }
    };

*/