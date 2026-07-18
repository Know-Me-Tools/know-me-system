// TJ-ARCH-MOB-001 compliant
import 'package:flutter/widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

/// Flat 2.0 screen frame. Regions are separated by filled backgrounds only.
class KnowMeScreen extends StatelessWidget {
  const KnowMeScreen({
    required this.title,
    required this.child,
    this.trailing = const [],
    super.key,
  });

  final String title;
  final Widget child;
  final List<Widget> trailing;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return shad.Scaffold(
      backgroundColor: colors.background,
      headerBackgroundColor: colors.secondary,
      headers: [
        SafeArea(
          bottom: false,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(20, 14, 16, 14),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: const TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                ...trailing,
              ],
            ),
          ),
        ),
      ],
      child: child,
    );
  }
}
