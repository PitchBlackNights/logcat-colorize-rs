# logcat-colorize (Rust Port)

A Rust rewrite of [carlonluca/logcat-colorize](https://github.com/carlonluca/logcat-colorize), the command-line tool that colorizes Android `adb logcat` output.

This version removes the Boost dependency, and aims to keeps the original behavior, implemented as a single fast Rust binary.

![logcat-colorize example](/assets/example.png)

## Features

- Colorizes `adb logcat` output by log level (`V`, `D`, `I`, `W`, `E`, `F`)
- Supports `threadtime`, `time`, `brief`, `process`, and `tag` formats
- Regex-based highlighting (`-s`, `--spotlight`)
- Option to ignore unrecognized lines (`-i`, `--ignore`)
- Lists ANSI color codes (`--list-ansi`)
- (UNIMPLEMENTED) Respects color environment variables

## Build

```bash
git clone https://github.com/chmouel/logcat-colorize
cd logcat-colorize
cargo build --release
````

Binary is created at `target/release/logcat-colorize`.

## Usage

Pipe from `adb logcat`:

```bash
adb logcat -v threadtime | target/release/logcat-colorize
```

Highlight text:

```bash
adb logcat -v time | target/release/logcat-colorize -s 'ERROR|FATAL'
```

Show color palette:

```bash
target/release/logcat-colorize --list-ansi
```

## Custom Colors (UNIMPLEMENTED)

Each color can be overridden with an ANSI escape sequence in an environment variable.

```bash
export LOGCAT_COLORIZE_ID_ERROR='^[1;48;5;160;38;5;15m'  # white on red
export LOGCAT_COLORIZE_MSG_INFO='^[0;38;5;34m'           # green text
```

Available variables:

```plaintext
LOGCAT_COLORIZE_ID_{DEBUG,VERBOSE,INFO,WARNING,ERROR,FATAL}
LOGCAT_COLORIZE_MSG_{DEBUG,VERBOSE,INFO,WARNING,ERROR,FATAL}
LOGCAT_COLORIZE_TID_PID
```

## License

Apache License 2.0 — same as the original.

Original authors: Bruno Braga, Luca Carlon
Rust port maintained independently.
