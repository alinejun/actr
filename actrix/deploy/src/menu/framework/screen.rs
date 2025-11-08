//! Screen management utilities

use std::io::{self, Write};

/// Clear screen utility
pub struct Screen;

impl Screen {
    /// Clear the screen using the most appropriate method
    pub fn clear() {
        if std::env::var("NO_CLEAR_SCREEN").is_ok() {
            // Just add some padding if clearing is disabled
            println!("\n\n");
            return;
        }

        // Try different clearing methods for maximum compatibility
        if Self::try_ansi_clear() {
            return;
        }

        // Fallback: try using the 'clear' command
        if Self::try_command_clear() {
            return;
        }

        // Final fallback: just add padding
        println!("\n\n");
    }

    /// Try to clear using ANSI escape codes
    fn try_ansi_clear() -> bool {
        match std::env::var("TERM") {
            Ok(term)
                if term.contains("xterm") || term.contains("screen") || term.contains("tmux") =>
            {
                // Use ANSI escape codes for supported terminals
                print!("\x1B[2J\x1B[1;1H");
                let _ = io::stdout().flush();
                true
            }
            _ => false,
        }
    }

    /// Try to clear using system command
    fn try_command_clear() -> bool {
        #[cfg(unix)]
        {
            if let Ok(mut child) = std::process::Command::new("clear").spawn() {
                let _ = child.wait();
                return true;
            }
        }

        #[cfg(windows)]
        {
            if let Ok(mut child) = std::process::Command::new("cls").spawn() {
                let _ = child.wait();
                return true;
            }
        }

        false
    }
}
