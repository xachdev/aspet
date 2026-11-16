//! Version information — the single source of truth.
//!
//! Cargo requires SemVer (`0.0.1-a01`); git tags and UCI use `0.0.1a01`.
//! Only Cargo.toml is edited; everything else derives from it.

/// Engine name.
pub const NAME: &str = "Aspet";

/// Version as Cargo/SemVer spells it, e.g. `0.0.1-a01`.
pub const SEMVER: &str = env!("CARGO_PKG_VERSION");

/// Tag/UCI style version, e.g. `0.0.1a01` (SemVer dash removed).
pub fn short() -> String {
    SEMVER.replace('-', "")
}

/// Full identification line, later used for the UCI `id name` response.
pub fn full_name() -> String {
    format!("{NAME} {}", short())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_and_tag_versions_agree() {
        assert_eq!(SEMVER, "0.0.1-a01");
        assert_eq!(short(), "0.0.1a01");
        assert_eq!(full_name(), "Aspet 0.0.1a01");
    }
}
