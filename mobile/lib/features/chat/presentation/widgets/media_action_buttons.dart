// TJ-ARCH-MOB-001 compliant — presentation; calls the service, not the plugins.
//
// Save and Share for media stored on disk. Lives in the app rather than in
// gen_ui_widgets because it depends on share_plus and path_provider, and the
// widget package must stay plugin-free so a web build can consume it.
import 'package:flutter/material.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../data/media_actions.dart';

/// Save and (where supported) Share, for a file in app-owned storage.
///
/// Returns a list so it can be spread straight into a block header beside the
/// existing Copy button.
List<Widget> buildMediaActionButtons({
  required String path,
  required String fileName,
  required String mime,
}) => [
  _MediaActionButton(
    icon: Icons.download_outlined,
    // Named for what it does to THIS file, so a screen reader moving through a
    // transcript of several images can tell the buttons apart.
    label: 'Save $fileName',
    onPressed: (context) async {
      final result = await const MediaActions().save(
        path: path,
        fileName: fileName,
      );
      if (context.mounted) _announce(context, result);
    },
  ),
  // Absent, not disabled, where the platform cannot share files: share_plus
  // 13.3.0 has no Linux file support and falls back to mailto, which cannot
  // carry an attachment. A control that silently does nothing is worse than
  // no control.
  if (MediaActions.canShareFiles)
    _MediaActionButton(
      icon: Icons.ios_share,
      label: 'Share $fileName',
      onPressed: (context) async {
        final result = await const MediaActions().share(
          path: path,
          fileName: fileName,
          mime: mime,
        );
        if (context.mounted) _announce(context, result);
      },
    ),
];

/// Tell the user what happened, in text — never by colour alone.
void _announce(BuildContext context, MediaActionResult result) {
  final message = switch (result) {
    MediaActionSaved(:final message) => message,
    // Dismissing the share sheet is a normal choice, not a failure, and saying
    // nothing at all leaves the user unsure whether the tap registered.
    MediaActionShared(dismissed: true) => 'Sharing cancelled.',
    MediaActionShared() => 'Shared.',
    MediaActionFailed(:final message) => message,
  };
  ScaffoldMessenger.maybeOf(context)
      ?.showSnackBar(SnackBar(content: Text(message)));
}

class _MediaActionButton extends StatelessWidget {
  const _MediaActionButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final Future<void> Function(BuildContext context) onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Semantics(
      button: true,
      label: label,
      excludeSemantics: true,
      child: IconButton(
        tooltip: label,
        // 48px: the WCAG 2.5.8 floor is 24px and the recommended touch target
        // is 44px. Matches the other icon buttons in this app.
        constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
        onPressed: () => onPressed(context),
        icon: Icon(icon, size: 18, color: colors.mutedForeground),
      ),
    );
  }
}
