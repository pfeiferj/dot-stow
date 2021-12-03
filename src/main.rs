extern crate yaml_rust;
extern crate dirs;

use glob::glob;
use std::path::PathBuf;
use std::os::unix::fs;
use std::fs::canonicalize;
use yaml_rust::YamlLoader;
use dirs::home_dir;

fn main() {
    use clap::{load_yaml, App};

    let yaml = load_yaml!("clap.yml");
    let m = App::from(yaml).get_matches();
    match m.value_of("source") {
        Some(source) => {
            match m.value_of("target") {
                Some(target) => handle_source_and_target(source, target),
                _ => (),
            };
            ()
        },
        _ => {
            handle_yaml();
        },
    };
}

fn handle_yaml()
{
    let stow_yaml_str = std::fs::read_to_string(".stow.yml")
        .expect("No .stow.yml file, please specify a source and target or for help use --help.");

    let stow_yaml = &YamlLoader::load_from_str(&stow_yaml_str).unwrap()[0];
    let mappings = stow_yaml["mappings"].as_vec()
        .expect("could not parse sources");
    for mapping in mappings {
        let source = mapping["source"].as_str()
            .expect("could not parse source of mapping");
        let target = mapping["target"].as_str()
            .expect("could not target source of mapping");
        handle_source_and_target(source, target);
    }
}

fn handle_source_and_target(source: &str, target: &str)
{
    let expanded_source = match source.chars().nth(0).expect("Source must not be blank") {
        '~' => {
            let stripped_source = source.strip_prefix("~").expect("String manipulation didn't succeed").to_owned();
            let home_path = home_dir().expect("Couldn't find home directory").to_owned();
            let home_path_str = home_path.into_os_string().into_string().unwrap().as_str().to_owned();
            let expanded_source = [home_path_str, stripped_source].join("");
            expanded_source
        },
        _ => String::from(source)
    };
    let expanded_target = match target.chars().nth(0).expect("Source must not be blank") {
        '~' => {
            let stripped_target = target.strip_prefix("~").expect("String manipulation didn't succeed").to_owned();
            let home_path = home_dir().expect("Couldn't find home directory").to_owned();
            let home_path_str = home_path.into_os_string().into_string().unwrap().as_str().to_owned();
            let expanded_target = [home_path_str, stripped_target].join("");
            expanded_target
        },
        _ => String::from(source)
    };
    println!("{}", ["linking ", expanded_source.as_str(), " -> ", expanded_target.as_str(), ":"].join(""));
    let source_glob = [expanded_source.as_str(), "/**/*"].join("");

    for entry in glob(&source_glob).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                match link_file(expanded_source.as_str(), &path, expanded_target.as_str()) {
                    Ok(_) => (),
                    Err(e) => eprintln!("    \x1b[93m{}\x1b[0m", e)
                }
                ()
            },
            Err(e) => println!("{:?}", e),
        }
    }
}

fn strip_source(source_prefix: &str, source: &PathBuf) -> PathBuf
{
    match source.strip_prefix(source_prefix) {
        Ok(p) => p.to_path_buf(),
        Err(_) => PathBuf::from(source),
    }
}
fn combine_paths(start: &PathBuf, end: &PathBuf) -> PathBuf
{
    let mut target_path = PathBuf::from(start);
    target_path.push(end);

    target_path
}

fn link_file(source_prefix: &str, source: &PathBuf, target: &str)  -> std::io::Result<()> {
    if !source.is_dir() {
        let source_stripped = strip_source(source_prefix, &source);
        let target_path = combine_paths(&PathBuf::from(target), &source_stripped);
        println!("{}", [
            "  * ",
            &source.display().to_string(),
            " -> ",
            &target_path.display().to_string()
        ].join(""));
        let mut target_parent = PathBuf::from(&target_path);
        target_parent.pop();
        std::fs::create_dir_all(target_parent)?;
        let canonical_source_path = canonicalize(source)?;
        fs::symlink(canonical_source_path, target_path)?;
    }
    Ok(())
}
