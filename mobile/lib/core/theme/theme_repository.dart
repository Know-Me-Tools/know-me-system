// TJ-ARCH-MOB-001 compliant
// Theme persistence boundary: providers depend on this repository; only this
// repository reaches the Rust FFI facade.
import '../../bridge/rust_bridge_provider.dart' as bridge;

class ThemeRepository {
  const ThemeRepository();

  Future<String> load() => bridge.getThemeMode();

  Future<void> save(String mode) => bridge.setThemeMode(mode);
}
