//! larp-jv-android — Android frontend para larp.jv

pub mod platform;
pub mod render;
pub mod input;
pub mod native;
pub mod ui;

pub use platform::callback::JniBridge;
pub use platform::app::AndroidAppState;
pub use render::surface::AndroidSurface;
pub use input::raw::InputHandler;