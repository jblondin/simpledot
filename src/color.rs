#[derive(Debug)]
pub enum Color {
    Rgb(Rgb),
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Hsv { h: f64, s: f64, v: f64 },
    Name(ColorName),
}

#[derive(Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

fn convert_hex(s: &str) -> u8 {
    u8::from_str_radix(s, 16).expect("hexcode_to_rgb expects well-formed RGB hex codes")
}

/// hexcode-to-rgb converter. panics on malformed RGB hex codes
pub fn hexcode_to_rgb(s: &str) -> Rgb {
    Rgb {
        r: convert_hex(&s[0..2]),
        g: convert_hex(&s[2..4]),
        b: convert_hex(&s[4..6]),
    }
}

#[derive(Debug)]
pub struct ColorName {
    scheme: ColorScheme,
    name: String,
}

#[derive(Debug)]
pub enum ColorScheme {
    X11,
    Svg,
    Brewer(BrewerScheme),
}

#[derive(Debug)]
pub enum BrewerScheme {}
