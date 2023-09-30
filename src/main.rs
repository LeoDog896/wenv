use std::ffi::OsString;

use winreg::RegValue;
use winreg::enums::*;
use winreg::RegKey;
use anyhow::Result;

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
    
    for (name, value) in cur_ver.enum_values().map(|x| x.unwrap()) {
        let value = match value.vtype {
            REG_SZ | REG_EXPAND_SZ => format!("{}", value),
            _ => unimplemented!("unimplemented type: {:?}", value.vtype)
        };
        println!("{}: {}", name, value);
    }

    Ok(())
}
