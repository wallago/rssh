mod feeds;
mod inbox;
mod saved;
mod settings;

pub mod prelude {
    pub use super::feeds::*;
    pub use super::inbox::*;
    pub use super::saved::*;
    pub use super::settings::*;
}
