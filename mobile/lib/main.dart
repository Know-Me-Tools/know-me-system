// TJ-ARCH-MOB-001 compliant
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'app/router.dart';
import 'bridge/rust_bridge_provider.dart';
import 'core/theme/tokens.dart';
import 'core/theme/theme_mode_provider.dart';
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

class AppRoot extends ConsumerWidget {
  const AppRoot({super.key});
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return shad.ShadcnApp.router(
      title: 'KnowMe',
      debugShowCheckedModeBanner: false,
      theme: KnowMeTheme.light,
      darkTheme: KnowMeTheme.dark,
      themeMode: ref.watch(themeModeProvider),
      routerConfig: appRouter,
      builder: (context, child) => StartupGate(
        child: child ?? const SizedBox.shrink(),
      ),
    );
  }
}
