#[allow(unused)]
pub mod color {
    pub const F_BLACK: &str = "30";
    pub const F_RED: &str = "31";
    pub const F_GREEN: &str = "32";
    pub const F_YELLOW: &str = "33";
    pub const F_BLUE: &str = "34";
    pub const F_PURPLE: &str = "35";
    pub const F_CYAN: &str = "36";
    pub const F_GREY: &str = "37";
    pub const FB_BLACK: &str = "90";
    pub const FB_RED: &str = "91";
    pub const FB_GREEN: &str = "92";
    pub const FB_YELLOW: &str = "93";
    pub const FB_BLUE: &str = "94";
    pub const FB_PURPLE: &str = "95";
    pub const FB_CYAN: &str = "96";
    pub const F_WHITE: &str = "97";
    pub const F_DEFAULT: &str = "39";

    pub const B_BLACK: &str = "40";
    pub const B_RED: &str = "41";
    pub const B_GREEN: &str = "42";
    pub const B_YELLOW: &str = "43";
    pub const B_BLUE: &str = "44";
    pub const B_PURPLE: &str = "45";
    pub const B_CYAN: &str = "46";
    pub const B_GREY: &str = "47";
    pub const BB_BLACK: &str = "100";
    pub const BB_RED: &str = "101";
    pub const BB_GREEN: &str = "102";
    pub const BB_YELLOW: &str = "103";
    pub const BB_BLUE: &str = "104";
    pub const BB_PURPLE: &str = "105";
    pub const BB_CYAN: &str = "106";
    pub const B_WHITE: &str = "107";
    pub const B_DEFAULT: &str = "49";
}

#[allow(unused)]
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
        super::ansi::color::B_DEFAULT,
        super::ansi::color::F_DEFAULT,
    )
}

pub fn list_ansi() {
    let fgs: [&str; 17] = [
        color::F_BLACK,
        color::F_RED,
        color::F_GREEN,
        color::F_YELLOW,
        color::F_BLUE,
        color::F_PURPLE,
        color::F_CYAN,
        color::F_GREY,
        color::FB_BLACK,
        color::FB_RED,
        color::FB_GREEN,
        color::FB_YELLOW,
        color::FB_BLUE,
        color::FB_PURPLE,
        color::FB_CYAN,
        color::F_WHITE,
        color::F_DEFAULT,
    ];
    let bgs: [&str; 17] = [
        color::B_BLACK,
        color::B_RED,
        color::B_GREEN,
        color::B_YELLOW,
        color::B_BLUE,
        color::B_PURPLE,
        color::B_CYAN,
        color::B_GREY,
        color::BB_BLACK,
        color::BB_RED,
        color::BB_GREEN,
        color::BB_YELLOW,
        color::BB_BLUE,
        color::BB_PURPLE,
        color::BB_CYAN,
        color::B_WHITE,
        color::B_DEFAULT,
    ];
    let attrs: [&str; 7] = [
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
                let seq: Seq = Seq::new(at, bg, fg);
                print!(
                    "{}^[{};{};{}m{}\x20",
                    seq.as_str(),
                    at,
                    bg,
                    fg,
                    reset().as_str()
                );
            }
            println!();
        }
    }
}
