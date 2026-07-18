// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../features/chat/presentation/screens/chat_screen.dart';
import '../features/memory/presentation/screens/memory_screen.dart';
import '../features/overview/presentation/screens/overview_screens.dart';

/// The four KnowMe-slice tabs, in shell order. Adding a destination here is the
/// only edit needed to surface a feature — labels/icons/paths stay in lockstep.
const _tabs = <(String, IconData, String)>[
  ('/home', Icons.home_outlined, 'Home'),
  ('/chat', Icons.chat_bubble_outline, 'Chat'),
  ('/hands', Icons.back_hand_outlined, 'Hands'),
  ('/memory', Icons.memory, 'Memory'),
  ('/models', Icons.developer_board_outlined, 'Models'),
  ('/settings', Icons.tune, 'Settings'),
];

final appRouter = GoRouter(
  initialLocation: '/home',
  routes: [
    ShellRoute(
      builder: (context, state, child) => _Shell(child: child),
      routes: [
        GoRoute(path: '/home', builder: (_, __) => const HomeScreen()),
        GoRoute(path: '/chat', builder: (_, __) => const ChatScreen()),
        GoRoute(path: '/hands', builder: (_, __) => const HandsScreen()),
        GoRoute(path: '/memory', builder: (_, __) => const MemoryScreen()),
        GoRoute(path: '/models', builder: (_, __) => const ModelsScreen()),
        GoRoute(path: '/settings', builder: (_, __) => const SettingsScreen()),
      ],
    ),
  ],
);

class _Shell extends StatelessWidget {
  const _Shell({required this.child});
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final location = GoRouterState.of(context).matchedLocation;
    final index = _tabs.indexWhere((t) => location.startsWith(t.$1));
    final colors = shad.Theme.of(context).colorScheme;
    final selected = ValueKey(_tabs[index < 0 ? 0 : index].$1);
    return shad.Scaffold(
      footerBackgroundColor: colors.secondary,
      footers: [
        SafeArea(
          top: false,
          child: shad.NavigationBar(
            selectedKey: selected,
            backgroundColor: colors.secondary,
            alignment: shad.NavigationBarAlignment.spaceEvenly,
            labelType: shad.NavigationLabelType.selected,
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
            onSelected: (key) {
              if (key is ValueKey<String>) context.go(key.value);
            },
            children: [
              for (final (path, icon, label) in _tabs)
                shad.NavigationItem(
                  key: ValueKey(path),
                  label: Text(label),
                  child: Icon(icon),
                ),
            ],
          ),
        ),
      ],
      child: child,
    );
  }
}
