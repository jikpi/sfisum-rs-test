pub enum TextColor {
    Red,
    Green,
    //Blue,
    //Yellow,
    //Magenta,
    Cyan,
    //White,
    //Black,
    BrightRed,
    BrightGreen,
    BrightBlue,
    BrightYellow,
    BrightMagenta,
    //BrightCyan,
    //BrightWhite,
    //Gray,
}

pub fn colorize_txt(color: TextColor, text: &str) -> String {
    let code = match color {
        //TextColor::Black => "30",
        TextColor::Red => "31",
        TextColor::Green => "32",
        //TextColor::Yellow => "33",
        //TextColor::Blue => "34",
        //TextColor::Magenta => "35",
        TextColor::Cyan => "36",
        //TextColor::White => "37",
        //TextColor::Gray => "90",
        TextColor::BrightRed => "91",
        TextColor::BrightGreen => "92",
        TextColor::BrightYellow => "93",
        TextColor::BrightBlue => "94",
        TextColor::BrightMagenta => "95",
        //TextColor::BrightCyan => "96",
        //TextColor::BrightWhite => "97",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}
