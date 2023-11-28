/// Subcommand Wizards for missing arguments.
use super::{
    colored::Colorize,
    continue_prompt, input_fmt,
    resource::{print_file_list, quit},
    ConfigToolInstallMethod,
};

use std::error::Error;

pub fn init_cmd_wizard() -> String {
    let filename = questionprint!("Enter a name for your project:");
    infoprint!("Your file will be created as {}.zzz.yaml.", filename);
    continue_prompt();
    let filename_f = format!("{}.zzz.yaml", filename);
    filename_f
}

pub fn add_cmd_wizard() -> Result<(String, String, ConfigToolInstallMethod), Box<dyn Error>> {
    match print_file_list(0) {
        Ok(res) => {
            let depname = questionprint!("Dependancy name:");
            let md = questionprint!("Install Method:\n\t1: Zipped Source File\n\t2: Git Link");
            let mut method: ConfigToolInstallMethod = ConfigToolInstallMethod::LINKZIP;
            match md.as_str() {
                "1" => method = ConfigToolInstallMethod::LINKZIP,
                "2" => method = ConfigToolInstallMethod::GIT,
                _ => {
                    quit(4);
                }
            }
            Ok((res.2, depname, method))
        }

        Err(..) => Err("ERR".into()),
    }
}

pub fn remove_cmd_wizard() -> Result<(String, String), Box<dyn Error>> {
    match print_file_list(0) {
        Ok(res) => {
            let depname = questionprint!("Dependancy name:");
            Ok((res.2, depname))
        }

        Err(..) => Err("ERR".into()),
    }
}
