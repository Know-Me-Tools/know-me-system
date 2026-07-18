// TJ-ARCH-MOB-001 compliant
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'app/router.dart';
import 'bridge/rust_bridge_provider.dart';
import 'features/startup/presentation/screens/startup_gate.dart';
import 'shared/providers/entity_transport.dart';
import 'shared/providers/data_dir.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Color(0x00000000),
      systemNavigationBarColor: Color(0x00000000),
    ),
  );
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);

  final dir = await getApplicationDocumentsDirectory();
  dataDirOverride = dir.path;
  await initRustBridge(dataDir: dir.path);

  runApp(
    ProviderScope(
      overrides: [
        // Wire PEM's transport seam to the FFI-backed adapter. Everything above
        // this override reaches Rust through the canonical path.
        entityTransportProvider.overrideWithValue(const FrbEntityTransport()),
      ],
      child: const AppRoot(),
    ),
  );
}

class AppRoot extends StatelessWidget {
  const AppRoot({super.key});
  @override
  Widget build(BuildContext context) {
    final dark = shad.LegacyColorSchemes.darkZinc().copyWith(
      background: () => const Color(0xFF0B0F14),
      foreground: () => const Color(0xFFE8EDF3),
      card: () => const Color(0xFF1C2535),
      cardForeground: () => const Color(0xFFE8EDF3),
      primary: () => const Color(0xFFFF6A3D),
      primaryForeground: () => const Color(0xFF0B0F14),
      secondary: () => const Color(0xFF161D29),
      secondaryForeground: () => const Color(0xFFE8EDF3),
      muted: () => const Color(0xFF253044),
      mutedForeground: () => const Color(0xFFA7B0BC),
      accent: () => const Color(0xFF202B40),
      accentForeground: () => const Color(0xFFE8EDF3),
      border: () => const Color(0x00000000),
      input: () => const Color(0xFF161D29),
      ring: () => const Color(0xFFFF6A3D),
    );
    final light = shad.LegacyColorSchemes.lightZinc().copyWith(
      background: () => const Color(0xFFF7F7F8),
      foreground: () => const Color(0xFF0B0F14),
      card: () => const Color(0xFFFFFFFF),
      cardForeground: () => const Color(0xFF0B0F14),
      primary: () => const Color(0xFFE04E28),
      primaryForeground: () => const Color(0xFFFFFFFF),
      secondary: () => const Color(0xFFFAFBFC),
      secondaryForeground: () => const Color(0xFF0B0F14),
      muted: () => const Color(0xFFF2F4F7),
      mutedForeground: () => const Color(0xFF4B5563),
      accent: () => const Color(0xFFF2F4F7),
      accentForeground: () => const Color(0xFF0B0F14),
      border: () => const Color(0x00000000),
      input: () => const Color(0xFFFAFBFC),
      ring: () => const Color(0xFFE04E28),
    );
    return shad.ShadcnApp.router(
      title: 'KnowMe',
      debugShowCheckedModeBanner: false,
      theme: shad.ThemeData(colorScheme: light, radius: 0.75),
      darkTheme: shad.ThemeData(colorScheme: dark, radius: 0.75),
      themeMode: shad.ThemeMode.system,
      routerConfig: appRouter,
      builder: (context, child) => StartupGate(
        child: child ?? const SizedBox.shrink(),
      ),
    );
  }
}
