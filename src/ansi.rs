pub struct Ansi;

impl Ansi {
    // Reset
    pub const RESET: &'static str = "\x1b[0m";

    // Styles
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const ITALIC: &'static str = "\x1b[3m";
    pub const UNDERLINE: &'static str = "\x1b[4m";
    pub const BLINK: &'static str = "\x1b[5m";
    pub const REVERSE: &'static str = "\x1b[7m";
    pub const HIDDEN: &'static str = "\x1b[8m";
    pub const STRIKETHROUGH: &'static str = "\x1b[9m";

    // Foreground Colors
    pub const FG_BLACK: &'static str = "\x1b[30m";
    pub const FG_RED: &'static str = "\x1b[31m";
    pub const FG_GREEN: &'static str = "\x1b[32m";
    pub const FG_YELLOW: &'static str = "\x1b[33m";
    pub const FG_BLUE: &'static str = "\x1b[34m";
    pub const FG_MAGENTA: &'static str = "\x1b[35m";
    pub const FG_CYAN: &'static str = "\x1b[36m";
    pub const FG_WHITE: &'static str = "\x1b[37m";

    // Bright Foreground Colors
    pub const FG_BRIGHT_BLACK: &'static str = "\x1b[90m";
    pub const FG_BRIGHT_RED: &'static str = "\x1b[91m";
    pub const FG_BRIGHT_GREEN: &'static str = "\x1b[92m";
    pub const FG_BRIGHT_YELLOW: &'static str = "\x1b[93m";
    pub const FG_BRIGHT_BLUE: &'static str = "\x1b[94m";
    pub const FG_BRIGHT_MAGENTA: &'static str = "\x1b[95m";
    pub const FG_BRIGHT_CYAN: &'static str = "\x1b[96m";
    pub const FG_BRIGHT_WHITE: &'static str = "\x1b[97m";

    // Background Colors
    pub const BG_BLACK: &'static str = "\x1b[40m";
    pub const BG_RED: &'static str = "\x1b[41m";
    pub const BG_GREEN: &'static str = "\x1b[42m";
    pub const BG_YELLOW: &'static str = "\x1b[43m";
    pub const BG_BLUE: &'static str = "\x1b[44m";
    pub const BG_MAGENTA: &'static str = "\x1b[45m";
    pub const BG_CYAN: &'static str = "\x1b[46m";
    pub const BG_WHITE: &'static str = "\x1b[47m";

    // Bright Background Colors
    pub const BG_BRIGHT_BLACK: &'static str = "\x1b[100m";
    pub const BG_BRIGHT_RED: &'static str = "\x1b[101m";
    pub const BG_BRIGHT_GREEN: &'static str = "\x1b[102m";
    pub const BG_BRIGHT_YELLOW: &'static str = "\x1b[103m";
    pub const BG_BRIGHT_BLUE: &'static str = "\x1b[104m";
    pub const BG_BRIGHT_MAGENTA: &'static str = "\x1b[105m";
    pub const BG_BRIGHT_CYAN: &'static str = "\x1b[106m";
    pub const BG_BRIGHT_WHITE: &'static str = "\x1b[107m";
}
