pub mod behavior;
pub mod content;
pub mod inventory;
pub mod scene;
#[cfg(feature = "workbook")]
pub mod workbook;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
