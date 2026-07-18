// TJ-ARCH-MOB-001 compliant
// Platform data directory, resolved once in main() via path_provider (Rust has
// no portable way to ask the OS for it — see rust_bridge_provider.dart's
// runMigrations doc comment) and read by the startup sequence.
late final String dataDirOverride;
