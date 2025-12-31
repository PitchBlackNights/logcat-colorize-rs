// Copyright 2025 chmouel
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod ansi;
mod logcat;
mod theme;

use crate::theme::{Theme, make_theme};
use clap::Parser;
use regex::Regex;
use std::{
    io::{self, IsTerminal},
    sync::LazyLock,
};

#[derive(Parser, Debug)]
#[command(version, disable_help_flag = true)]
struct Args {
    /// Does not output non-matching data
    #[arg(short, long)]
    ignore: bool,

    /// Highlight pattern in the output, value as REGEXP (e.g. -s '\bWORD\b')
    #[arg(short, long)]
    spotlight: Option<String>,

    /// Prints this help
    #[arg(short, long)]
    help: bool,

    /// List available ansi escape codes to format the output
    #[arg(long)]
    list_ansi: bool,
}

static HELP_TEXT: LazyLock<String> = LazyLock::new(|| -> String {
    format!(
        "{name} v{ver}

A simple tool to colorize Android adb logcat output.
Pipe adb into this program. Supports Tag, Process, Brief, Time, and ThreadTime.

Usage:
  adb logcat [options] | {name} [options]

Options:
  -i, --ignore        do not output non-matching lines
  -h, --help          show help
  -s, --spotlight RE  highlight regex pattern in output

Examples:
  adb logcat | {name}
  adb -s emulator-5556 logcat -v time System.err:V *:S | {name}
  adb logcat -v time | egrep -i '(sensor|wifi)' | {name}

Authors: Bruno Braga, Luca Carlon
Adapted to Rust: Chmouel Boudjnah
Bugs: https://github.com/chmouel/logcat-colorize-rs/issues",
        name = env!("CARGO_PKG_NAME"),
        ver = env!("CARGO_PKG_VERSION")
    )
});

fn main() -> io::Result<()> {
    let args: Args = Args::parse();

    if args.help {
        println!("{}", *HELP_TEXT);
        return Ok(());
    }
    if args.list_ansi {
        ansi::list_ansi();
        return Ok(());
    }

    if io::stdin().is_terminal() {
        println!("{}", *HELP_TEXT);
        return Ok(());
    }

    let theme: Theme = make_theme();
    let spotlight_re: Option<Regex> = args
        .spotlight
        .as_ref()
        .and_then(|s: &String| Regex::new(&format!("({})", s)).ok());

    logcat::format_with(&theme, spotlight_re, args.ignore)
}
