/// Command Execution
// Local imports
use crate::{
    helper::{
        check_arg_len,
        colored::Colorize,
        input_fmt, read_file,
        resource::{calculate_hash, continue_prompt, read_file_gpath},
        usage_and_quit, verbose_check, verbose_info_print, Tool, UniConfig,
    },
    list, LOADCMD,
};

use crate::helper::errors::{invalid_file_error, missing_file_error};

// std imports
use std::{env, error::Error, fs, fs::File, io::BufReader, process::Command};

pub fn list_exec(v_file: File, filepath: String, way: usize) -> Result<(), Box<dyn Error>> {
    let reader: BufReader<File> = BufReader::new(v_file);
    // Parse the YAML
    let config: Result<UniConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);
    match config {
        Err(_) => {
            invalid_file_error(&filepath);
            Err("Invalid Config".into())
        }

        Ok(config) => match way {
            1 => {
                infoprint!("'{}' requires the following dependancies:", filepath);
                let mut num = 1;
                for tool in config.deps.tools {
                    println!("  {0}: {1}", num, tool.name);
                    num += 1;
                }
                Ok(())
            }

            _ => {
                infoprint!("Dependancies for {}:", filepath);
                let mut num = 1;
                for tool in config.deps.tools {
                    println!("  {0}: {1}", num, tool.name);
                    num += 1;
                }
                Ok(())
            }
        },
    }
}

pub fn add_exec(filepath: &String, depname: &String) -> Result<(), Box<dyn Error>> {
    let link = questionprint!("Enter link for '{}':", depname);
    match read_file_gpath(&filepath) {
        Ok(v_file) => {
            let config: Result<UniConfig, serde_yaml::Error> = serde_yaml::from_reader(&v_file.0);
            let mut conf_f = config.unwrap();

            let n_tool: Tool = Tool {
                name: depname.to_string(),
                link,
            };
            let mut tool_to_add: Vec<Tool> = vec![n_tool];
            //let to_w = conf_f.deps.tools.append(&mut tool_to_add);
            conf_f.deps.tools.append(&mut tool_to_add);
            let f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(v_file.1)
                .expect("Couldn't open file");
            serde_yaml::to_writer(f, &conf_f).unwrap();
        }
        Err(file) => {
            missing_file_error(&file.1);
        }
    };

    successprint!("'{0}' added to {1}", depname, &filepath);

    Ok(())
}

pub fn load_exec(
    v_file: File,
    filepath: String,
    mut env_cmds: Vec<String>,
    mut home_dir: Result<String, env::VarError>,
    global_opts: &[bool],
) -> Result<(Vec<String>, u64), Box<dyn Error>> {
    let reader: BufReader<File> = BufReader::new(v_file);
    // Parse the YAML into DepConfig struct
    let config: Result<UniConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);
    match config {
        Err(_) => {
            invalid_file_error(&filepath);
            Err("Invalid Config".into())
        }
        Ok(config) => {
            infoprint!("Getting dependancies from file: '{}'", filepath);
            let hashname = calculate_hash(&config.project.name);
            //println!("{}", hash_string(&config.project.name));
            for tool in config.deps.tools {
                let _ = tool_install(tool, hashname, &mut env_cmds, &mut home_dir, global_opts);
            }
            let result = (env_cmds, hashname);
            Ok(result)
        }
    }
}

pub fn load_deps(
    argsv: Vec<String>,
    env_cmds: &[String],
    home_dir: Result<String, env::VarError>,
    global_opts: &[bool],
) -> Result<(Vec<String>, u64), Box<dyn Error>> {
    if check_arg_len(argsv.clone(), 2) {
        usage_and_quit(LOADCMD.name, "Missing Filename!");
        return Err("Bad File".into());
    } else {
        let _ = list(argsv.clone(), 1);
        if global_opts[2] {
            infoprint!("This action will download the above, and run any tasks included.");
        }
        continue_prompt(global_opts);
        let _: Result<(Vec<String>, u64), ()> = match read_file(&argsv, 2, LOADCMD) {
            Ok(v_file) => {
                let result =
                    load_exec(v_file.0, v_file.1, env_cmds.to_vec(), home_dir, global_opts);
                return result;
                //Ok(result)
            }
            Err(file) => {
                missing_file_error(&file.1);
                Err(())
            }
        };
    }
    Err("Bad File".into())
}

fn tool_install(
    tool: Tool,
    hashname: u64,
    env_cmds: &mut Vec<String>,
    home_dir: &mut Result<String, env::VarError>,
    global_opts: &[bool],
) -> Result<(), Box<dyn Error>> {
    env_cmds.push(tool.name.clone());
    verbose_info_print(
        format!("Installing {0} from {1}", tool.name, tool.link),
        global_opts,
    );
    let link = tool.link;
    let link_str = link.to_string();
    if cfg!(windows) {
        let dir_loc = format!(
            "{0}\\.unify\\bins\\{1}\\",
            home_dir.as_mut().unwrap(),
            hashname
        );
        match fs::create_dir_all(&dir_loc) {
            Ok(..) => {
                let namef = format!("{0}{1}", dir_loc, tool.name);
                let args: Vec<&str> = vec!["/C", "curl", &link_str, "--output", &namef, "--silent"];
                //println!("{:?}", args);

                let status = Command::new("cmd").args(args).status()?;
                if status.success() {
                    let args2: Vec<&str> = vec!["/C", "chmod", "a+x", &namef];
                    let status2 = Command::new("cmd").args(args2).status()?;
                    if status2.success() {
                        verbose_info_print(format!("'{}' installed", tool.name), global_opts);
                        Ok(())
                    } else {
                        errprint!("Error grabbing: '{}'", tool.name);
                        continue_prompt(global_opts);
                        Err("Error grabbing".into())
                    }
                    //infoprint!("Command '{}' executed successfully", command);
                } else {
                    errprint!("Error grabbing: '{}'", tool.name);
                    continue_prompt(global_opts);
                    Err("Error grabbing".into())
                }
            }
            Err(..) => {
                errprint!("Error creating dir");
                Err("Error creating dir".into())
            }
        }
    } else {
        let dir_loc = format!("{0}/.unify/bins/{1}/", home_dir.as_mut().unwrap(), hashname);
        match fs::create_dir_all(&dir_loc) {
            Ok(..) => {
                let link_str_f = link_str.to_string();
                let namef = format!("{0}{1}", dir_loc, tool.name);
                let args: Vec<&str> = vec!["-c", "/usr/bin/curl", &link_str_f, "--output", &namef];
                let status = Command::new("bash").args(args).status()?;

                if status.success() {
                    let args2: Vec<&str> = vec!["-c", "chmod", "a+x", &namef];
                    let status2 = Command::new("bash").args(args2).status()?;
                    if status2.success() {
                        verbose_info_print(format!("'{}' installed", tool.name), global_opts);
                        Ok(())
                    } else {
                        errprint!("Error grabbing: '{}'", tool.name);
                        Err("Error grabbing".into())
                    }
                } else {
                    errprint!("Error grabbing: '{}'", tool.name);
                    Err("Error grabbing".into())
                }
            }
            Err(..) => {
                errprint!("Error creating dir");
                Err("Error creating dir".into())
            }
        }
    }
}

pub fn run_exec(
    v_file: File,
    filepath: String,
    global_opts: Vec<bool>,
) -> Result<(), Box<dyn Error>> {
    let reader: BufReader<File> = BufReader::new(v_file);
    // Parse the YAML into PluConfig struct
    let config: Result<UniConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);
    match config {
        Err(_) => {
            missing_file_error(&filepath);
            Err("Invalid Config".into())
        }

        Ok(config) => {
            let mut okcount: i32 = 0;
            let mut cmdcount: i32 = 0;
            // Execute commands in the 'run' section
            infoprint!("Running '{}': \n", filepath);
            for command in config.r#do.run {
                cmdcount += 1;
                let mut parts = command.split_whitespace();
                let program = parts.next().ok_or("Missing command")?;
                let args: Vec<&str> = parts.collect();
                let status = Command::new(program).args(args).status()?;
                if status.success() {
                    if verbose_check(&global_opts) {
                        infoprint!("Command '{}' executed successfully", command);
                    }
                    okcount += 1;
                } else {
                    errprint!("Error executing command: '{}'", command);
                }
            }
            if cmdcount == okcount {
                println!();
                successprint!("All tasks completed successfully");
                println!();
            }
            Ok(())
        }
    }
}

pub fn spin_exec(v_file: File, filepath: String, global_opts: Vec<bool>, home_dir: &mut Result<String, env::VarError>)  -> Result<(), String>{
    let reader: BufReader<File> = BufReader::new(v_file);
    // Parse the YAML into DepConfig struct
    let config: Result<UniConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);
    match config {
        Err(_) => {
            invalid_file_error(&filepath);
            Err("Invalid Config".into())
        }
        Ok(config) => {
            infoprint!("Getting dependancies from file: '{}'", filepath);
            let hashname = calculate_hash(&config.project.name);

            if cfg!(windows) {
                let dir_loc = format!(
                    "{0}\\.unify\\bins\\{1}\\",
                    home_dir.as_mut().unwrap(), hashname
                );
                Ok(())
            } else {
                let dir_loc = format!("{0}/.unify/cache/{1}/", home_dir.as_mut().unwrap(), hashname);
                Ok(())
            }
        }
    }
}
