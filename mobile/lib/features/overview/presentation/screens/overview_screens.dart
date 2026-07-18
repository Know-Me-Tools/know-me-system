// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart' show Icons;
import 'package:flutter/widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

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

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) => const _OverviewScreen(
        item: OverviewItem(
          title: 'Settings',
          eyebrow: 'Privacy and appearance',
          heading: 'Your data. Your choices.',
          body:
              'KnowMe follows the system light or dark theme and keeps local conversations in the shared Rust data layer.',
          icon: Icons.tune,
        ),
      );
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
