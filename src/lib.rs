//! Root library file exposing the Hexagonal Architecture modules.
//!
//! # Architecture
//! - `core`: Domain logic and business rules (Pure Rust).
//! - `ports`: Interfaces (Traits) defining interaction contracts.
//! - `adapters`: Infrastructure implementations (FS, I/O).

pub mod core;
pub mod ports;
pub mod adapters;

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the library structure allows compilation and testing.
    #[test]
    fn architecture_sanity_check() {
        // This test ensures that the test runner is correctly picking up the lib root.
        let status = true;
        assert!(status, "The test environment should be operational");
    }
}