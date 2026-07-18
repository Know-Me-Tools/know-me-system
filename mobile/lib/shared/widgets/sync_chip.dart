// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:prometheus_entity_management/prometheus_entity_management.dart';

import '../providers/sync_status_provider.dart';
import '../../core/theme/tokens.dart';

/// A small status chip driven by the Rust SyncStatus stream. Presentational;
/// the exhaustive switch over the sealed SyncStatus is a compile-time contract.
class SyncChip extends ConsumerWidget {
  const SyncChip({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(syncStatusProvider);
    final (label, color) = switch (status.value) {
      SyncOffline() || null => ('offline', T.textTertiary),
      SyncSyncing(:final pendingWrites) => (
          'syncing · $pendingWrites',
          T.amber
        ),
      SyncLive() => ('live', T.green),
      SyncError() => ('error', T.red),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 6,
            height: 6,
            decoration: BoxDecoration(color: color, shape: BoxShape.circle),
          ),
          const SizedBox(width: 6),
          Text(label, style: T.uiMd.copyWith(color: color)),
        ],
      ),
    );
  }
}
