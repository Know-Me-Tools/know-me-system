import 'package:flutter/material.dart';

/// The id of the synthetic on-device provider. Rendered as "On-device".
const String kOnDeviceProviderId = 'on-device';

/// A provider → model cascade that writes a `{ provider_id, model_id }` route
/// (or `null` to inherit the scope below it). Shared across every model-selection
/// tier: global run policy, per-agent policy, and per-conversation policy.
///
/// Providers come from the configured-provider catalog (`snapshot.providers`);
/// each entry carries a `models` list. The on-device provider needs no key and
/// is always selectable; cloud providers are selectable when configured.
class ModelRoutePicker extends StatelessWidget {
  const ModelRoutePicker({
    super.key,
    required this.providers,
    required this.value,
    required this.onChanged,
    this.inheritLabel = 'Runtime default',
    this.enabled = true,
  });

  /// Catalog providers (each a `Map` with `id`, `displayName`, `models`).
  final List<Map<String, dynamic>> providers;

  /// Current route as `{ 'providerId': ..., 'modelId': ... }`, or null to inherit.
  final Map<String, String>? value;

  /// Emits the new route, or null when the user clears the selection.
  final ValueChanged<Map<String, String>?> onChanged;

  /// Label for the "no override" option (e.g. "Runtime default", "Inherit").
  final String inheritLabel;

  final bool enabled;

  static const String _inherit = '__inherit__';

  List<Map<String, dynamic>> _selectableProviders() => providers
      .where(
        (provider) =>
            provider['configured'] == true ||
            provider['id'] == kOnDeviceProviderId,
      )
      .toList();

  List<Map<String, dynamic>> _modelsFor(String? providerId) {
    if (providerId == null) return const [];
    final provider = providers
        .where((item) => item['id']?.toString() == providerId)
        .firstOrNull;
    final raw = provider?['models'];
    if (raw is List) {
      return raw.whereType<Map>().map((m) => m.cast<String, dynamic>()).toList();
    }
    return const [];
  }

  String _providerLabel(Map<String, dynamic> provider) {
    if (provider['id'] == kOnDeviceProviderId) return 'On-device';
    return provider['displayName']?.toString() ?? provider['id'].toString();
  }

  @override
  Widget build(BuildContext context) {
    final selectable = _selectableProviders();
    final providerId = value?['providerId'];
    final modelId = value?['modelId'];
    final models = _modelsFor(providerId);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DropdownButtonFormField<String>(
          initialValue: providerId ?? _inherit,
          isExpanded: true,
          decoration: const InputDecoration(
            labelText: 'Provider',
            border: OutlineInputBorder(),
          ),
          items: [
            DropdownMenuItem(value: _inherit, child: Text(inheritLabel)),
            for (final provider in selectable)
              DropdownMenuItem(
                value: provider['id'].toString(),
                child: Text(_providerLabel(provider)),
              ),
          ],
          onChanged: !enabled
              ? null
              : (selected) {
                  if (selected == null || selected == _inherit) {
                    onChanged(null);
                    return;
                  }
                  final providerModels = _modelsFor(selected);
                  final first = providerModels.isNotEmpty
                      ? providerModels
                            .firstWhere(
                              (m) => m['enabled'] != false,
                              orElse: () => providerModels.first,
                            )['id']
                            .toString()
                      : '';
                  onChanged({'providerId': selected, 'modelId': first});
                },
        ),
        const SizedBox(height: 12),
        DropdownButtonFormField<String>(
          initialValue: (value != null && models.isNotEmpty) ? modelId : null,
          isExpanded: true,
          decoration: InputDecoration(
            labelText: 'Model',
            border: const OutlineInputBorder(),
            hintText: value == null ? 'Select a provider first' : 'Choose a model',
          ),
          items: [
            for (final model in models)
              DropdownMenuItem(
                value: model['id'].toString(),
                child: Text(
                  model['displayName']?.toString() ?? model['id'].toString(),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
          ],
          onChanged: (!enabled || value == null || models.isEmpty)
              ? null
              : (selected) {
                  if (selected == null) return;
                  onChanged({
                    'providerId': value!['providerId']!,
                    'modelId': selected,
                  });
                },
        ),
      ],
    );
  }
}
