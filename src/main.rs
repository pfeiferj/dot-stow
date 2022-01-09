use shellexpand;
use std::path::PathBuf;
use std::os::unix::fs as unix_fs;
use std::fs;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;
use std::process::Command;
use clap::{load_yaml, App};
use std::io::prelude::*;

enum CommandType {
    Pre,
    Post,
}

struct CommandOptionsPresent {
    source: bool,
    target: bool,
    init: bool
}

enum ExecutionType {
    CurrentFolder,
    SourceAndTarget,
    Init,
    Undefined,
}

fn main() {
    let yaml = load_yaml!("clap.yml");
    let mut app = App::from(yaml);
    let m = App::from(yaml).get_matches();
    let cmd_options_present = CommandOptionsPresent {
        source: m.is_present("source"),
        target: m.is_present("target"),
        init: m.is_present("init")
    };
    match get_command_type(cmd_options_present) {
        ExecutionType::CurrentFolder => handle_yaml(&mut app),
        ExecutionType::SourceAndTarget => {
            let sat = get_source_and_target(m);
            handle_source_and_target(sat.0.as_str(), sat.1.as_str(), false);
        },
        ExecutionType::Init => {
            match init() {
                Ok(_) => (),
                Err(_) => println!("Could not create files."),
            }
        },
        ExecutionType::Undefined => {
            println!("Unsupported arguments");
        },
    }
}

fn init() -> std::io::Result<()> {
    let install_file_str = include_str!("./init_files/install");
    let stow_file_str = include_str!("./init_files/.stow/.stow.yml");
    let install_sh_file_str = include_str!("./init_files/.stow/scripts/install.sh");
    let gitignore_file_str = include_str!("./init_files/.stow/.gitignore");


    if !std::path::Path::new(".stow").exists() {
        fs::create_dir(".stow")?;
    }

    if !std::path::Path::new(".stow/scripts").exists() {
        fs::create_dir(".stow/scripts")?;
    }

    let mut buffer = std::fs::OpenOptions::new().create(true).write(true).open("install")?;
    buffer.write_all(install_file_str.as_bytes())?;

    let mut buffer = std::fs::OpenOptions::new().create(true).write(true).open(".stow/scripts/install.sh")?;
    buffer.write_all(install_sh_file_str.as_bytes())?;

    if !std::path::Path::new(".stow/.stow.yml").exists() {
        fs::write(".stow/.stow.yml", stow_file_str)?;
    }

    if !std::path::Path::new(".stow/.gitignore").exists() {
        fs::write(".stow/.gitignore", gitignore_file_str)?;
    }
    Ok(())
}

fn get_source_and_target(matches: clap::ArgMatches) -> (String, String) {
    let source = matches.value_of("source")
        .expect("Argument matches must contain source.");
    let target = matches.value_of("target")
        .expect("Argument matches must contain target.");
    return ( String::from(source), String::from(target));
}

fn get_command_type(command_options: CommandOptionsPresent) -> ExecutionType {
    match command_options {
        CommandOptionsPresent { source: false, target: false, init: true } => ExecutionType::Init,
        CommandOptionsPresent { source: true, target: true, init: false } => ExecutionType::SourceAndTarget,
        CommandOptionsPresent { source: false, target: false, init: false } => ExecutionType::CurrentFolder,
        _ => ExecutionType::Undefined
    }
}

fn handle_yaml(app: &mut App)
{
    app.print_help()
        .expect("Failed to print help");
    let stow_yaml_str = match fs::read_to_string(".stow/.stow.yml") {
        Ok(str) => str,
        Err(_) => {
            app.print_help()
                .expect("Failed to print help");
            println!("\x1b[93m{}\x1b[0m", "\nNo .stow.yml found");
            std::process::exit(0);
        }
    };

    let stow_yaml = &YamlLoader::load_from_str(&stow_yaml_str).unwrap()[0];

    handle_script(CommandType::Pre, stow_yaml);   

    let mappings = stow_yaml["mappings"].as_vec()
        .expect("could not parse sources");
    for mapping in mappings {
        let source = mapping["source"].as_str()
            .expect("could not parse source of mapping");
        let target = mapping["target"].as_str()
            .expect("could not target source of mapping");
        let as_copy_opt = mapping["as_copy"].as_bool();
        let as_copy = match as_copy_opt {
            Some(x) => x, 
            None => false,
        };

        handle_script(CommandType::Post, mapping);   
        handle_source_and_target(source, target, as_copy);
        handle_script(CommandType::Post, mapping);   
    }

    handle_script(CommandType::Post, stow_yaml);   
}

fn handle_script(command_type: CommandType, yaml: &Yaml)
{
    let script_target_option = match command_type {
        CommandType::Pre => &yaml["pre_stow"],
        CommandType::Post => &yaml["post_stow"],
    }.as_str();
    match script_target_option {
        Some(target) => {
            let output = Command::new("sh")
                .arg(target)
                .output()
                .expect("Failed to execute pre_stow script.");
            print!("{}", String::from_utf8(output.stdout)
                .expect("stdout is utf-8 string."));
            print!("{}", String::from_utf8(output.stderr)
                .expect("stderr is utf-8 string."));
        },
        None => (),
    }
}

fn handle_source_and_target(source: &str, target: &str, as_copy: bool)
{
    let expanded_source = shellexpand::tilde(source);
    let expanded_target = shellexpand::tilde(target);
    let action_prefix = match as_copy {
        true => "Copying ",
        false => "Linking "
    };
    println!("{}", [action_prefix, &expanded_source, " -> ", &expanded_target, ":"].join(""));
    let source_glob = [&expanded_source, "/**/*"].join("");

    for entry in glob::glob(&source_glob).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if as_copy {
                    match copy_file(&expanded_source, &path, &expanded_target) {
                        Ok(_) => (),
                        Err(e) => eprintln!("    \x1b[93m{}\x1b[0m", e)
                    }

                } else {
                    match link_file(&expanded_source, &path, &expanded_target) {
                        Ok(_) => (),
                        Err(e) => eprintln!("    \x1b[93m{}\x1b[0m", e)
                    }
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
        fs::create_dir_all(target_parent)?;
        let canonical_source_path = fs::canonicalize(source)?;
        unix_fs::symlink(canonical_source_path, target_path)?;
    }
    Ok(())
}

fn copy_file(source_prefix: &str, source: &PathBuf, target: &str)  -> std::io::Result<()> {
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
        fs::create_dir_all(target_parent)?;
        let canonical_source_path = fs::canonicalize(source)?;

        fs::copy(canonical_source_path, target_path)?;
    }
    Ok(())
}
