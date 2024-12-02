#![no_std]

/// The ANSI escape sequence `\x1b[20h` is used to enable a specific terminal mode, known as
/// **line feed/new line (LF/NL) mode** in the DEC terminal standard (`DECSET 20`).
///
/// ### ANSI Escape Sequence
/// - `\x1b` represents the ANSI escape character (ESC), used for controlling terminal behavior.
/// - `[20h` is a command sequence that enables a specific terminal mode.
/// - Together, `\x1b[20h` enables a mode in the terminal.
///
/// ### Purpose
/// - **LF/NL Mode**: In this mode, sending a newline character (`\n`) moves the cursor to the
///   beginning of the next line, combining the effects of both a **line feed (LF)** and a
///   **carriage return (CR)**.
/// - This ensures consistent and readable line breaks in terminal output, especially in environments
///   like the Wokwi console.
///
/// ### Usage in Wokwi
/// - When using the Wokwi simulator, this escape sequence is used to ensure proper formatting of
///   serial output in the simulated UART terminal.
/// - Without this mode, explicit `\r\n` sequences might be needed for consistent line endings.
///
/// ### Compatibility
/// - Supported by terminal emulators that recognize DEC VT-series commands (e.g., xterm or similar).
/// - Terminals that do not support this command will likely ignore it.
///
/// ### Disabling the Mode
/// - The corresponding sequence to disable this mode is `\x1b[20l`.
///
/// ### Example
/// This sequence can be conditionally included in Rust code for use in Wokwi:
/// ```rust
/// #[cfg(feature = "wokwi")]
/// print!("\x1b[20h");
/// ```
/// In this case, the `wokwi` feature flag can be enabled when building for the Wokwi simulator.
#[macro_export]
macro_rules! enable_lf_nl_mode {
    () => {
        print!("\x1b[20h");
    };
}
