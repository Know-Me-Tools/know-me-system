// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart' show Icons;
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../core/theme/theme_mode_provider.dart';
import '../../../../shared/widgets/knowme_screen.dart';
import '../../domain/overview_item.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) => const _OverviewScreen(
        item: OverviewItem(
          title: 'Home',
          eyebrow: 'Your private intelligence',
          heading: 'Good to see you.',
          body:
              'Continue a conversation, revisit a memory, or ask KnowMe to connect what matters.',
          icon: Icons.auto_awesome,
        ),
      );
}

class HandsScreen extends StatelessWidget {
  const HandsScreen({super.key});

  @override
  Widget build(BuildContext context) => const _OverviewScreen(
        item: OverviewItem(
          title: 'Hands',
          eyebrow: 'Skills and actions',
          heading: 'Put context to work.',
          body:
              'Approved skills and tool activity appear here, with every action visible and under your control.',
          icon: Icons.back_hand_outlined,
        ),
      );
}

class ModelsScreen extends StatelessWidget {
  const ModelsScreen({super.key});

  @override
  Widget build(BuildContext context) => const _OverviewScreen(
        item: OverviewItem(
          title: 'Models',
          eyebrow: 'Local by default',
          heading: 'Private intelligence, on device.',
          body:
              'KnowMe downloads and runs a compatible local model automatically. Cloud providers remain an explicit choice.',
          icon: Icons.memory,
        ),
      );
}

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final colors = shad.Theme.of(context).colorScheme;
    final selected = ref.watch(themeModeProvider);
    return KnowMeScreen(
      title: 'Settings',
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: colors.card,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Appearance',
                  style: TextStyle(
                    color: colors.foreground,
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  'Choose a WCAG AA light or dark palette, or match this device.',
                  style: TextStyle(color: colors.mutedForeground, height: 1.4),
                ),
                const SizedBox(height: 16),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    for (final option in const [
                      (shad.ThemeMode.system, Icons.brightness_auto, 'System'),
                      (
                        shad.ThemeMode.light,
                        Icons.light_mode_outlined,
                        'Light'
                      ),
                      (shad.ThemeMode.dark, Icons.dark_mode_outlined, 'Dark'),
                    ])
                      _ThemeModeButton(
                        icon: option.$2,
                        label: option.$3,
                        selected: selected == option.$1,
                        onPressed: () => ref
                            .read(themeModeProvider.notifier)
                            .select(option.$1),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _ThemeModeButton extends StatelessWidget {
  const _ThemeModeButton({
    required this.icon,
    required this.label,
    required this.selected,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final child = Semantics(
      selected: selected,
      label: '$label theme',
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [Icon(icon, size: 18), const SizedBox(width: 8), Text(label)],
      ),
    );
    return selected
        ? shad.PrimaryButton(onPressed: onPressed, child: child)
        : shad.GhostButton(onPressed: onPressed, child: child);
  }
}

class _OverviewScreen extends StatelessWidget {
  const _OverviewScreen({required this.item});

  final OverviewItem item;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return KnowMeScreen(
      title: item.title,
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Container(
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: colors.card,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  width: 48,
                  height: 48,
                  decoration: BoxDecoration(
                    color: colors.accent,
                    borderRadius: BorderRadius.circular(14),
                  ),
                  child: Icon(item.icon, color: colors.primary),
                ),
                const SizedBox(height: 28),
                Text(
                  item.eyebrow.toUpperCase(),
                  style: TextStyle(
                    color: colors.primary,
                    fontSize: 11,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 1.2,
                  ),
                ),
                const SizedBox(height: 10),
                Text(
                  item.heading,
                  style: const TextStyle(
                    fontSize: 30,
                    fontWeight: FontWeight.w700,
                    height: 1.1,
                  ),
                ),
                const SizedBox(height: 12),
                Text(
                  item.body,
                  style: TextStyle(
                    color: colors.mutedForeground,
                    fontSize: 16,
                    height: 1.5,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
