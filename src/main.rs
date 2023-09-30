use std::ffi::OsString;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::Write;
use tabwriter::TabWriter;
use winreg::enums::*;
use winreg::RegKey;
use winreg::RegValue;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let cur_ver = hklm.open_subkey("Environment")?;

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
                            "(too long - run {})",
                            format!("`wenv show {}`", name.underline()).purple()
                        )
                        .yellow()
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
