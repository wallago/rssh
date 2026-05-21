use std::hash::{DefaultHasher, Hash, Hasher};

pub fn string_to_color(s: &str) -> String {
    // generate hash form string (not randomly)
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish(); // hash from srting

    // get HLS code
    let hue = (hash % 360) as f64; // Hue (0–360°): position on the color wheel (red → yellow → green → cyan → blue → magenta → red)
    let saturation = 0.65; // Saturation (0–1): grayness vs. vividness
    let lightness = 0.60; //Lightness (0–1): black → color → white
    let chroma = (1.0 - (2.0 * lightness - 1.0)) * saturation; // Chroma is the magnitude of colorfulness
    let sector = hue / 60.0; // Divid Hue in sector (in degree)
    let x = chroma * (1.0 - ((sector % 2.0) - 1.0).abs()); // Second color channel's value within the sector
    let m = lightness - chroma / 2.0; // Lightness offset added to all three channels equally
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let to = |v: f64| ((v + m) * 255.0).round() as u8; // Scales to 0–255

    format!("#{:02x}{:02x}{:02x}", to(r), to(g), to(b))
}
