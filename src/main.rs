use std::ffi::OsString;
use std::path::Path;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::Write;
use tabwriter::TabWriter;
use winreg::enums::*;
use winreg::RegKey;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Show raw (non-tty) output.
    #[arg(long)]
    raw: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List out specific envs(s)
    Show {
        /// Specific envs(s) to list out
        paths: Vec<OsString>,
    },
}

fn pretty_registry(hklm: RegKey, subkey: &str) -> Result<()> {
    let cur_ver = hklm.open_subkey(subkey)?;

    if let Some((w, _)) = term_size::dimensions() {
        let mut tw = TabWriter::new(vec![]);

        let longest_name_length = cur_ver
            .enum_values()
            .map(|x| x.unwrap())
            .map(|(name, _)| name.len())
            .max()
            .unwrap();

        for (name, value) in cur_ver.enum_values().map(|x| x.unwrap()) {
            let value = match value.vtype {
                REG_SZ | REG_EXPAND_SZ => format!("{}", value),
                _ => unimplemented!("unimplemented type: {:?}", value.vtype),
            };
            tw.write(
                format!(
                    "{}\t{}\n",
                    name.blue(),
                    if longest_name_length + 2 + value.len() > w {
                        format!(
                            "({}>{} characters - run {}{}",
                            value.len(),
                            w - longest_name_length - 2,
                            format!("`wenv show {}`", name).purple(),
                            ")".red()
                        )
                        .red()
                        .to_string()
                    } else {
                        value.green().to_string()
                    }
                )
                .as_bytes(),
            )?;
        }

        tw.flush().unwrap();

        println!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
    }

    Ok(())
}

fn registry(hklm: RegKey, subkey: &str) -> Result<()> {
    let cur_ver = hklm.open_subkey(subkey)?;

    for (name, value) in cur_ver.enum_values().map(|x| x.unwrap()) {
        println!("{}: {}", name, value);
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_tty = atty::is(atty::Stream::Stdout) && !cli.raw;

    match cli.command {
        None => {
            let hklm = RegKey::predef(HKEY_CURRENT_USER);

            if is_tty {
                pretty_registry(hklm, "Environment")?;
            } else {
                registry(hklm, "Environment")?;
            }
        }
        Some(Commands::Show { paths }) => {
            let hklm = RegKey::predef(HKEY_CURRENT_USER);
            let cur_ver = hklm.open_subkey("Environment")?;

            let values: Vec<(&OsString, OsString)> = paths
                .iter()
                .map(|x| (x, cur_ver.get_value(x).unwrap()))
                .collect::<Vec<_>>();

            for (key, value) in values {
                let key = key.to_str().unwrap();
                println!("{}", key.blue());

                if key == "Path" {
                    let mut problem_count = 0;
                    let path = value.to_str().unwrap();
                    let path = path.split(";").collect::<Vec<_>>();
                    for path_str in path {
                        let path = Path::new(path_str);

                        // check if path exists
                        if !path.exists() {
                            println!("{}", format!("  {} {}", path_str, "(does not exist)").red());
                            problem_count += 1;
                            continue;
                        }

                        println!("  {}", path.to_str().unwrap());
                    }

                    if problem_count > 0 {
                        println!("{}", format!("{} problems found", problem_count).red());
                    } else {
                        println!("{}", "0 problems found".green());
                    }

                    continue;
                }

                println!("{:?}", value);
            }
        }
    }

    Ok(())
}
