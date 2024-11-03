#![no_std]

#[cfg(feature = "other_names")]
mod container {
    include!("other_names.rs");
}

#[cfg(not(feature = "other_names"))]
mod container {
    pub const MS_YELLOW: &str = "Yellow";
    pub const MR_PURPLE: &str = "Purple";
}

pub fn ms_yellow() -> &'static str {
    container::MS_YELLOW
}

pub fn mr_purple() -> &'static str {
    container::MR_PURPLE
}
