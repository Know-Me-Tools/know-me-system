/// This is copied from Cargokit (which is the official way to use it currently)
/// Details: https://fzyzcjy.github.io/flutter_rust_bridge/manual/integrate/builtin

import 'dart:io';

import 'package:logging/logging.dart';
import 'package:path/path.dart' as path;

import 'artifacts_provider.dart';
import 'builder.dart';
import 'environment.dart';
import 'options.dart';
import 'target.dart';

final log = Logger('build_gradle');

class BuildGradle {
  BuildGradle({required this.userOptions});

  final CargokitUserOptions userOptions;

  Future<void> build() async {
    final targets = Environment.targetPlatforms.map((arch) {
      final target = Target.forFlutterName(arch);
      if (target == null) {
        throw Exception(
            "Unknown darwin target or platform: $arch, ${Environment.darwinPlatformName}");
      }
      return target;
    }).toList();

    final environment = BuildEnvironment.fromEnvironment(isAndroid: true);
    final provider =
        ArtifactProvider(environment: environment, userOptions: userOptions);
    final artifacts = await provider.getArtifacts(targets);

    for (final target in targets) {
      final libs = artifacts[target]!;
      final outputDir = path.join(Environment.outputDir, target.android!);
      Directory(outputDir).createSync(recursive: true);

      for (final lib in libs) {
        if (lib.type == AritifactType.dylib) {
          File(lib.path).copySync(path.join(outputDir, lib.finalFileName));
        }
      }

      _copyLibcxxShared(target, outputDir);
    }
  }

  /// The NDK's clang links c++_shared by default for any C++ dependency that
  /// doesn't explicitly request a different runtime (RocksDB, onig, whisper.cpp,
  /// ...); cargokit otherwise never bundles it, so the app installs but crashes
  /// on first launch with "libc++_shared.so not found" the moment gen_ui_ffi's
  /// dylib is dlopen'd. Statically linking instead (CXXSTDLIB=c++_static) isn't
  /// a safe alternative here: at least one dependency's build.rs (RocksDB,
  /// crates.io surrealdb-librocksdb-sys) always emits `dylib=$CXXSTDLIB`
  /// verbatim regardless of the env var's value, producing a nonsensical
  /// "dylib=c++_static" link request and a mixed-runtime ABI mismatch
  /// (dlopen: "cannot locate symbol _ZNSt9exceptionD2Ev"). Bundling the real
  /// shared object is the only combination that works with every dependency.
  void _copyLibcxxShared(Target target, String outputDir) {
    final hostArch = Platform.isMacOS
        ? "darwin-x86_64"
        : (Platform.isLinux ? "linux-x86_64" : "windows-x86_64");
    // The NDK sysroot's per-arch lib dirs mostly match the Rust target triple,
    // except 32-bit ARM: NDK ships "arm-linux-androideabi", Rust's target is
    // "armv7-linux-androideabi".
    final sysrootArchDir =
        target.rust == 'armv7-linux-androideabi' ? 'arm-linux-androideabi' : target.rust;
    final libcxxPath = path.joinAll([
      Environment.sdkPath,
      'ndk',
      Environment.ndkVersion,
      'toolchains',
      'llvm',
      'prebuilt',
      hostArch,
      'sysroot',
      'usr',
      'lib',
      sysrootArchDir,
      'libc++_shared.so',
    ]);
    final libcxxFile = File(libcxxPath);
    if (!libcxxFile.existsSync()) {
      throw Exception('libc++_shared.so not found at $libcxxPath');
    }
    libcxxFile.copySync(path.join(outputDir, 'libc++_shared.so'));
  }
}
