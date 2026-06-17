use std::hash::{DefaultHasher, Hash, Hasher};

/// Generale prelude Module
pub mod prelude {
    pub use crate::utils::*;
}

/// Generate color from a given string
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

/// Paint the Android system status + navigation bars with the app background
/// so the OS chrome doesn't show its own (usually black/white) color.
///
/// This is the JNI interop the project allows: it reaches the running Activity
/// via `ndk_context`, grabs its `Window`, and calls the same setters you'd use
/// from Kotlin. It must run on the UI thread — calling it from a Dioxus effect
/// (which runs on tao's event-loop / Android main thread) satisfies that.
///
/// NOTE: this path is `cfg(target_os = "android")`-gated, so host `cargo check`
/// / `just lint` do not compile it — verify with `just run` on a device.
#[cfg(target_os = "android")]
pub fn theme_system_bars() -> anyhow::Result<()> {
    use dioxus::mobile::wry::prelude::dispatch;
    dispatch(|env, activity, _webview| {
        let result: anyhow::Result<()> = (|| {
            // Get the window
            let window = env
                .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;

            // Set window flags (for example, set keep screen on)
            const FLAG_KEEP_SCREEN_ON: i32 = 128;
            env.call_method(&window, "addFlags", "(I)V", &[FLAG_KEEP_SCREEN_ON.into()])?;

            // Use dark icons for status bar
            let insets_controller = env
                .call_method(
                    &window,
                    "getInsetsController",
                    "()Landroid/view/WindowInsetsController;",
                    &[],
                )?
                .l()?;

            if !insets_controller.is_null() {
                let appearance_flag = 0x00000008; // APPEARANCE_LIGHT_STATUS_BARS
                env.call_method(
                    &insets_controller,
                    "setSystemBarsAppearance",
                    "(II)V",
                    &[appearance_flag.into(), appearance_flag.into()],
                )?;
            }

            // Set status bar color
            let color = 0xFF0D0D0Du32 as i32; // ARGB
            env.call_method(&window, "setStatusBarColor", "(I)V", &[color.into()])?;

            // Set navigation bar color
            env.call_method(&window, "setNavigationBarColor", "(I)V", &[color.into()])?;

            Ok(())
        })();
        if let Err(err) = result {
            eprintln!("theme_system_bars failed: {err}");
        }
    });
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn theme_system_bars() -> anyhow::Result<()> {
    Ok(())
}
