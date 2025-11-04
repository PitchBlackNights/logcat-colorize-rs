// main.rs
// logcat-colorize (Rust port)
// License: Apache-2.0 (same as original)

use std::io::{self, BufRead, Write};

use ansi::{Seq, attr, color};
use atty::Stream;
use clap::Parser;
use once_cell::sync::Lazy;
use regex::Regex;

const NAME: &str = "logcat-colorize";
const VERSION: &str = "0.10.1-rs";

#[derive(Parser, Debug)]
#[command(name = NAME, version = VERSION, disable_help_flag = true)]
struct Args {
    /// Does not output non-matching data
    #[arg(short = 'i', long = "ignore")]
    ignore: bool,

    /// Highlight pattern in the output, value as REGEXP (e.g. -s '\bWORD\b')
    #[arg(short = 's', long = "spotlight")]
    spotlight: Option<String>,

    /// Prints this help
    #[arg(short = 'h', long = "help")]
    help: bool,

    /// List available ansi escape codes to format the output
    #[arg(long = "list-ansi")]
    list_ansi: bool,
}

static HELP_TEXT: Lazy<String> = Lazy::new(|| {
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
      --list-ansi     list ANSI escape combinations

Environment overrides (use ^[attr;bg;fgm):
  LOGCAT_COLORIZE_ID_DEBUG
  LOGCAT_COLORIZE_ID_VERBOSE
  LOGCAT_COLORIZE_ID_INFO
  LOGCAT_COLORIZE_ID_WARNING
  LOGCAT_COLORIZE_ID_ERROR
  LOGCAT_COLORIZE_ID_FATAL
  LOGCAT_COLORIZE_MSG_DEBUG
  LOGCAT_COLORIZE_MSG_VERBOSE
  LOGCAT_COLORIZE_MSG_INFO
  LOGCAT_COLORIZE_MSG_WARNING
  LOGCAT_COLORIZE_MSG_ERROR
  LOGCAT_COLORIZE_MSG_FATAL
  LOGCAT_COLORIZE_TID_PID

Examples:
  adb logcat | {name}
  adb -s emulator-5556 logcat -v time System.err:V *:S | {name}
  adb logcat -v time | egrep -i '(sensor|wifi)' | {name}

Authors: Bruno Braga, Luca Carlon
Bugs: https://github.com/carlonluca/logcat-colorize/issues",
        name = NAME,
        ver = VERSION
    )
});

mod ansi {
    pub mod color {
        pub const FBLACK: &str = "30";
        pub const FRED: &str = "31";
        pub const FGREEN: &str = "32";
        pub const FYELLOW: &str = "33";
        pub const FBLUE: &str = "34";
        pub const FPURPLE: &str = "35";
        pub const FCYAN: &str = "36";
        pub const FWHITE: &str = "97";
        pub const FDEFAULT: &str = "39";

        pub const BBLACK: &str = "40";
        pub const BRED: &str = "41";
        pub const BGREEN: &str = "42";
        pub const BYELLOW: &str = "43";
        pub const BBLUE: &str = "44";
        pub const BPURPLE: &str = "45";
        pub const BCYAN: &str = "46";
        pub const BWHITE: &str = "47";
        pub const BDEFAULT: &str = "49";
    }

    pub mod attr {
        pub const RESET: &str = "0";
        pub const BOLD: &str = "1";
        pub const FAINT: &str = "2";
        pub const UNDERLINE: &str = "4";
        pub const SLOWBLINK: &str = "5";
        pub const FASTBLINK: &str = "6";
        pub const REVERSE: &str = "7";
    }

    #[derive(Clone, Debug)]
    pub struct Seq {
        cached: String,
    }

    impl Seq {
        pub fn new(attr: &str, bg: &str, fg: &str) -> Self {
            Self {
                cached: format!("\x1b[{};{};{}m", attr, bg, fg),
            }
        }
        pub fn as_str(&self) -> &str {
            &self.cached
        }
    }

    pub fn reset() -> Seq {
        Seq::new(
            super::ansi::attr::RESET,
            super::ansi::color::BDEFAULT,
            super::ansi::color::FDEFAULT,
        )
    }
}

#[derive(Clone, Debug, Default)]
struct Logcat {
    date: String,
    level: String, // V D I W E F
    tag: String,
    process: String, // pid
    message: String,
    thread: String, // tid
}

#[derive(Clone)]
struct Theme {
    id_verbose: Seq,
    id_debug: Seq,
    id_info: Seq,
    id_warning: Seq,
    id_error: Seq,
    id_fatal: Seq,
    msg_verbose: Seq,
    msg_debug: Seq,
    msg_info: Seq,
    msg_warning: Seq,
    msg_error: Seq,
    msg_fatal: Seq,
    tid_pid: Seq,
    reset: Seq,
}

fn themed_default() -> Theme {
    use ansi::Seq;

    macro_rules! seq256 {
        ($attr:expr, $bg:expr, $fg:expr) => {
            Seq::new($attr, &format!("48;5;{}", $bg), &format!("38;5;{}", $fg))
        };
    }

    Theme {
        id_verbose: seq256!("1", 24, 15),
        id_debug: seq256!("1", 33, 15),
        id_info: seq256!("1", 34, 15),
        id_warning: seq256!("1", 178, 0),
        id_error: seq256!("1", 160, 15),
        id_fatal: seq256!("1", 125, 15),

        msg_verbose: seq256!("0", 0, 37),
        msg_debug: seq256!("0", 0, 33),
        msg_info: seq256!("0", 0, 34),
        msg_warning: seq256!("0", 0, 178),
        msg_error: seq256!("0", 0, 160),
        msg_fatal: seq256!("0", 0, 125),

        tid_pid: seq256!("0", 236, 81),

        reset: ansi::reset(),
    }
}

// Regexes for formats
static RE_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([VDIWEF])/(.*?): (.*)$").unwrap());
static RE_PROCESS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([VDIWEF])\(([ 0-9]{1,})\) (.*) \(((.*?)?)\)$").unwrap());
static RE_BRIEF: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([VDIWEF])/(.*?)\(([ 0-9]{1,})\): (.*)$").unwrap());
static RE_TIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}):? ([VDIWEF])/(.*?)\(([ 0-9]{1,})\)\s*: (.*)$").unwrap()
});
static RE_THREADTIME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3})\s*([0-9]{1,})\s*([0-9]{1,}) ([VDIWEF]) (.*?): (.*)$").unwrap()
});

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FormatKind {
    ThreadTime,
    Time,
    Brief,
    Process,
    Tag,
}

fn parse_line(line: &str) -> Option<(FormatKind, Logcat)> {
    if let Some(c) = RE_THREADTIME.captures(line) {
        return Some((
            FormatKind::ThreadTime,
            Logcat {
                date: c[1].to_string(),
                process: c[2].trim().to_string(),
                thread: c[3].trim().to_string(),
                level: c[4].to_string(),
                tag: c[5].to_string(),
                message: c[6].to_string(),
            },
        ));
    }
    if let Some(c) = RE_TIME.captures(line) {
        return Some((
            FormatKind::Time,
            Logcat {
                date: c[1].to_string(),
                level: c[2].to_string(),
                tag: c[3].to_string(),
                process: c[4].trim().to_string(),
                message: c[5].to_string(),
                ..Default::default()
            },
        ));
    }
    if let Some(c) = RE_BRIEF.captures(line) {
        return Some((
            FormatKind::Brief,
            Logcat {
                level: c[1].to_string(),
                tag: c[2].to_string(),
                process: c[3].trim().to_string(),
                message: c[4].to_string(),
                ..Default::default()
            },
        ));
    }
    if let Some(c) = RE_PROCESS.captures(line) {
        return Some((
            FormatKind::Process,
            Logcat {
                level: c[1].to_string(),
                process: c[2].trim().to_string(),
                message: c[3].to_string(),
                tag: c.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                ..Default::default()
            },
        ));
    }
    if let Some(c) = RE_TAG.captures(line) {
        return Some((
            FormatKind::Tag,
            Logcat {
                level: c[1].to_string(),
                tag: c[2].to_string(),
                message: c[3].to_string(),
                ..Default::default()
            },
        ));
    }
    None
}

fn spot_if_needed(s: &str, spot: &Option<Regex>, spot_paint: &str, resume_seq: &str) -> String {
    if let Some(re) = spot {
        // Insert colored $1 then resume sequence
        let replacement = format!("{}$1{}", spot_paint, resume_seq);
        re.replace_all(s, replacement.as_str()).into_owned()
    } else {
        s.to_string()
    }
}

fn print_log(
    l: &Logcat,
    theme: &Theme,
    spot: &Option<Regex>,
    out: &mut impl Write,
) -> io::Result<()> {
    // Spotlight color: bold, red background, white fg
    let spot_seq = Seq::new(attr::RESET, color::BRED, color::FWHITE)
        .as_str()
        .to_string();

    // date
    if !l.date.is_empty() {
        let date_seq = Seq::new(attr::RESET, color::BDEFAULT, color::FPURPLE);
        let seg = spot_if_needed(&l.date, spot, &spot_seq, date_seq.as_str());
        write!(
            out,
            "{} {} {}",
            date_seq.as_str(),
            seg,
            theme.reset.as_str()
        )?;
    }

    // level sequences
    let (id_seq, msg_seq) = match l.level.as_str() {
        "D" => (&theme.id_debug, &theme.msg_debug),
        "V" => (&theme.id_verbose, &theme.msg_verbose),
        "I" => (&theme.id_info, &theme.msg_info),
        "W" => (&theme.id_warning, &theme.msg_warning),
        "E" => (&theme.id_error, &theme.msg_error),
        "F" => (&theme.id_fatal, &theme.msg_fatal),
        _ => (&theme.reset, &theme.reset),
    };

    if !l.level.is_empty() {
        if !std::ptr::eq(id_seq, &theme.reset) {
            write!(
                out,
                " {} {} {}",
                id_seq.as_str(),
                l.level,
                theme.reset.as_str()
            )?;
        }
        write!(out, " ")?;
    }

    // [pid/tid]
    if !l.process.is_empty() {
        let mut bracket = String::new();
        if l.thread.is_empty() {
            bracket.push_str(&format!("[{}]", l.process));
        } else {
            bracket.push_str(&format!("[{}/{}]", l.process, l.thread));
        }
        let seg = spot_if_needed(&bracket, spot, &spot_seq, theme.tid_pid.as_str());
        write!(
            out,
            "{}{}{}",
            theme.tid_pid.as_str(),
            seg,
            theme.reset.as_str()
        )?;
    }

    // tag
    if !l.tag.is_empty() {
        let tag_seq = Seq::new(attr::RESET, color::BDEFAULT, color::FWHITE);
        let seg = spot_if_needed(&l.tag, spot, &spot_seq, tag_seq.as_str());
        write!(out, "{} {}{}", tag_seq.as_str(), seg, theme.reset.as_str())?;
    }

    // message
    if !l.message.is_empty() {
        write!(out, " ")?;
        if !std::ptr::eq(msg_seq, &theme.reset) {
            write!(out, "{}", msg_seq.as_str())?;
        }
        let seg = spot_if_needed(&l.message, spot, &spot_seq, msg_seq.as_str());
        write!(out, "{seg}")?;
    }

    write!(out, "{}", theme.reset.as_str())?;
    writeln!(out)
}

fn list_ansi() {
    let fgs = [
        color::FDEFAULT,
        color::FBLACK,
        color::FRED,
        color::FGREEN,
        color::FYELLOW,
        color::FBLUE,
        color::FPURPLE,
        color::FCYAN,
        color::FWHITE,
    ];
    let bgs = [
        color::BDEFAULT,
        color::BBLACK,
        color::BRED,
        color::BGREEN,
        color::BYELLOW,
        color::BBLUE,
        color::BPURPLE,
        color::BCYAN,
        color::BWHITE,
    ];
    let attrs = [
        attr::RESET,
        attr::BOLD,
        attr::FAINT,
        attr::UNDERLINE,
        attr::SLOWBLINK,
        attr::FASTBLINK,
        attr::REVERSE,
    ];

    for (i, bg) in bgs.iter().enumerate() {
        println!("\nBackground {i}:");
        for fg in fgs {
            for at in attrs {
                let seq = Seq::new(at, bg, fg);
                print!(
                    "{}^[{};{};{}m{}\x20",
                    seq.as_str(),
                    at,
                    bg,
                    fg,
                    ansi::reset().as_str()
                );
            }
            println!();
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.help {
        println!("{}", *HELP_TEXT);
        return Ok(());
    }
    if args.list_ansi {
        list_ansi();
        return Ok(());
    }

    if atty::is(Stream::Stdin) {
        // No pipe. Show help.
        println!("{}", *HELP_TEXT);
        return Ok(());
    }

    let theme = themed_default();

    // Spotlight regex prepared once, applied at print time
    let spotlight_re = args
        .spotlight
        .as_ref()
        .and_then(|s| Regex::new(&format!("({})", s)).ok());

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut guessed_kind: Option<FormatKind> = None;

    for line in stdin.lock().lines() {
        let line = line?;
        if guessed_kind.is_none() {
            if let Some((kind, lc)) = parse_line(&line) {
                guessed_kind = Some(kind);
                let _ = print_log(&lc, &theme, &spotlight_re, &mut stdout);
                continue;
            } else if !args.ignore {
                writeln!(stdout, "{line}")?;
            }
            continue;
        }

        // Fast path using the already-guessed kind, with fallback once.
        let mut parsed: Option<Logcat> = None;
        match guessed_kind.unwrap() {
            FormatKind::ThreadTime => {
                if let Some(c) = RE_THREADTIME.captures(&line) {
                    parsed = Some(Logcat {
                        date: c[1].to_string(),
                        process: c[2].trim().to_string(),
                        thread: c[3].trim().to_string(),
                        level: c[4].to_string(),
                        tag: c[5].to_string(),
                        message: c[6].to_string(),
                    });
                }
            }
            FormatKind::Time => {
                if let Some(c) = RE_TIME.captures(&line) {
                    parsed = Some(Logcat {
                        date: c[1].to_string(),
                        level: c[2].to_string(),
                        tag: c[3].to_string(),
                        process: c[4].trim().to_string(),
                        message: c[5].to_string(),
                        ..Default::default()
                    });
                }
            }
            FormatKind::Brief => {
                if let Some(c) = RE_BRIEF.captures(&line) {
                    parsed = Some(Logcat {
                        level: c[1].to_string(),
                        tag: c[2].to_string(),
                        process: c[3].trim().to_string(),
                        message: c[4].to_string(),
                        ..Default::default()
                    });
                }
            }
            FormatKind::Process => {
                if let Some(c) = RE_PROCESS.captures(&line) {
                    parsed = Some(Logcat {
                        level: c[1].to_string(),
                        process: c[2].trim().to_string(),
                        message: c[3].to_string(),
                        tag: c.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                        ..Default::default()
                    });
                }
            }
            FormatKind::Tag => {
                if let Some(c) = RE_TAG.captures(&line) {
                    parsed = Some(Logcat {
                        level: c[1].to_string(),
                        tag: c[2].to_string(),
                        message: c[3].to_string(),
                        ..Default::default()
                    });
                }
            }
        }

        if let Some(lc) = parsed {
            let _ = print_log(&lc, &theme, &spotlight_re, &mut stdout);
        } else {
            // Fallback: try re-guess once, then print raw if still failing.
            if let Some((kind, lc)) = parse_line(&line) {
                guessed_kind = Some(kind);
                let _ = print_log(&lc, &theme, &spotlight_re, &mut stdout);
            } else if !args.ignore {
                writeln!(stdout, "{line}")?;
            }
        }
    }

    Ok(())
}
