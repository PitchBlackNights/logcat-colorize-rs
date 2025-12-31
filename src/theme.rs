use crate::ansi::{self, Seq};

#[derive(Clone)]
pub struct Theme {
    pub id_verbose: Seq,
    pub id_debug: Seq,
    pub id_info: Seq,
    pub id_warning: Seq,
    pub id_error: Seq,
    pub id_fatal: Seq,

    pub msg_verbose: Seq,
    pub msg_debug: Seq,
    pub msg_info: Seq,
    pub msg_warning: Seq,
    pub msg_error: Seq,
    pub msg_fatal: Seq,

    pub timestamp: Seq,
    pub tid_pid: Seq,
    pub tag: Seq,
    pub reset: Seq,
}

pub fn make_theme() -> Theme {
    macro_rules! seq {
        ($attr:ident, $bg:ident, $fg:ident) => {
            $crate::ansi::Seq::new(
                $crate::ansi::attr::$attr,
                $crate::ansi::color::$bg,
                $crate::ansi::color::$fg,
            )
        };
    }

    Theme {
        id_verbose: seq!(BOLD, B_CYAN, F_BLACK),
        id_debug: seq!(BOLD, B_BLUE, F_BLACK),
        id_info: seq!(BOLD, B_GREEN, F_BLACK),
        id_warning: seq!(BOLD, B_YELLOW, F_BLACK),
        id_error: seq!(BOLD, B_RED, F_BLACK),
        id_fatal: seq!(BOLD, B_BLACK, F_DEFAULT),

        msg_verbose: seq!(RESET, B_DEFAULT, F_CYAN),
        msg_debug: seq!(RESET, B_DEFAULT, F_BLUE),
        msg_info: seq!(RESET, B_DEFAULT, F_GREEN),
        msg_warning: seq!(RESET, B_DEFAULT, F_YELLOW),
        msg_error: seq!(RESET, B_DEFAULT, F_RED),
        msg_fatal: seq!(BOLD, B_DEFAULT, FB_RED),

        timestamp: seq!(RESET, B_DEFAULT, F_PURPLE),
        tid_pid: seq!(RESET, B_DEFAULT, F_PURPLE),
        tag: seq!(RESET, B_DEFAULT, F_DEFAULT),
        reset: ansi::reset(),
    }
}
