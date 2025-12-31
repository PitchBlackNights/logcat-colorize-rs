use crate::{
    ansi::{Seq, attr, color},
    theme::Theme,
};
use regex::Regex;
use std::{
    io::{self, BufRead},
    sync::LazyLock,
};

// Regexes for formats
pub static RE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([VDIWEF])/(.*?): (.*)$").unwrap());
pub static RE_PROCESS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([VDIWEF])\(([ 0-9]{1,})\) (.*) \(((.*?)?)\)$").unwrap());
pub static RE_BRIEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([VDIWEF])/(.*?)\(([ 0-9]{1,})\): (.*)$").unwrap());
pub static RE_TIME: LazyLock<Regex> = LazyLock::new(|| -> Regex {
    Regex::new(r"^([0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}):? ([VDIWEF])/(.*?)\(([ 0-9]{1,})\)\s*: (.*)$").unwrap()
});
pub static RE_THREADTIME: LazyLock<Regex> = LazyLock::new(|| -> Regex {
    Regex::new(r"^([0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3})\s*([0-9]{1,})\s*([0-9]{1,}) ([VDIWEF]) (.*?): (.*)$").unwrap()
});

#[derive(Clone, Debug, Default)]
pub struct Logcat {
    timestamp: String,
    level: String, // V D I W E F
    tag: String,
    process: String, // pid
    message: String,
    thread: String, // tid
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FormatKind {
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
                timestamp: c[1].to_string(),
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
                timestamp: c[1].to_string(),
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
                tag: c
                    .get(4)
                    .map(|m: regex::Match<'_>| m.as_str().to_string())
                    .unwrap_or_default(),
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
        let replacement: String = format!("{}$1{}", spot_paint, resume_seq);
        re.replace_all(s, replacement.as_str()).into_owned()
    } else {
        s.to_string()
    }
}

fn print_log(l: &Logcat, theme: &Theme, spot: &Option<Regex>) {
    // Spotlight color: bold, red background, white fg
    let spot_seq: String = Seq::new(attr::RESET, color::B_RED, color::F_WHITE)
        .as_str()
        .to_string();

    // Level colors
    let (id_seq, msg_seq) = match l.level.as_str() {
        "V" => (&theme.id_verbose, &theme.msg_verbose),
        "D" => (&theme.id_debug, &theme.msg_debug),
        "I" => (&theme.id_info, &theme.msg_info),
        "W" => (&theme.id_warning, &theme.msg_warning),
        "E" => (&theme.id_error, &theme.msg_error),
        "F" => (&theme.id_fatal, &theme.msg_fatal),
        _ => (&theme.reset, &theme.reset),
    };

    // Timestamp
    if !l.timestamp.is_empty() {
        let seg: String = spot_if_needed(&l.timestamp, spot, &spot_seq, theme.timestamp.as_str());
        print!(
            "{}{}{} ",
            theme.timestamp.as_str(),
            seg,
            theme.reset.as_str()
        );
    }

    // Level
    if !l.level.is_empty() {
        print!("{} {} {} ", id_seq.as_str(), l.level, theme.reset.as_str());
    }

    // [pid/tid]
    if !l.process.is_empty() {
        let bracket: String = if l.thread.is_empty() {
            format!("[{}]", l.process)
        } else {
            format!("[{}/{}]", l.process, l.thread)
        };
        let seg: String = spot_if_needed(&bracket, spot, &spot_seq, theme.tid_pid.as_str());
        print!("{}{}{} ", theme.tid_pid.as_str(), seg, theme.reset.as_str());
    }

    // Tag
    if !l.tag.is_empty() {
        let seg: String = spot_if_needed(&l.tag, spot, &spot_seq, theme.tag.as_str());
        print!("{}{}{} ", theme.tag.as_str(), seg, theme.reset.as_str());
    }

    // Message
    if !l.message.is_empty() {
        let seg: String = spot_if_needed(&l.message, spot, &spot_seq, msg_seq.as_str());
        print!("{}{}{} ", msg_seq.as_str(), seg, theme.reset.as_str());
    }

    println!();
}

pub fn format_with(theme: &Theme, spotlight_re: Option<Regex>, ignore: bool) -> io::Result<()> {
    let stdin: io::Stdin = io::stdin();
    let mut guessed_kind: Option<FormatKind> = None;

    for line in stdin.lock().lines() {
        let line: String = line?;
        if guessed_kind.is_none() {
            if let Some((kind, lc)) = parse_line(&line) {
                guessed_kind = Some(kind);
                print_log(&lc, theme, &spotlight_re);
                continue;
            } else if !ignore {
                println!("{}", line);
            }
            continue;
        }

        // Fast path using the already-guessed kind, with fallback once.
        let mut parsed: Option<Logcat> = None;
        match guessed_kind.unwrap() {
            FormatKind::ThreadTime => {
                if let Some(c) = RE_THREADTIME.captures(&line) {
                    parsed = Some(Logcat {
                        timestamp: c[1].to_string(),
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
                        timestamp: c[1].to_string(),
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
                        tag: c
                            .get(4)
                            .map(|m: regex::Match<'_>| m.as_str().to_string())
                            .unwrap_or_default(),
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
            print_log(&lc, theme, &spotlight_re);
        } else {
            // Fallback: try re-guess once, then print raw if still failing.
            if let Some((kind, lc)) = parse_line(&line) {
                guessed_kind = Some(kind);
                print_log(&lc, theme, &spotlight_re);
            } else if !ignore {
                println!("{}", line);
            }
        }
    }

    Ok(())
}
