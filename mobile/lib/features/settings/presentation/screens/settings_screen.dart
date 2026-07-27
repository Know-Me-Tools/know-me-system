// TJ-ARCH-MOB-001 compliant — responsive client over the Rust/UAR control plane.
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../../core/theme/theme_mode_provider.dart';
import '../../../../shared/widgets/knowme_screen.dart';
import '../providers/settings_notifier.dart';
import '../widgets/model_route_picker.dart';

/// The well-known id of the on-device local provider in the unified picker.
/// Mirrors the Rust `LOCAL_PROVIDER_ID`; selecting it routes the embedded
/// runtime to the on-device model rather than a cloud provider.
const _localProviderId = 'on-device';

enum _Section {
  providers('Model providers', Icons.cloud_outlined),
  runtime('Runtime & policy', Icons.hub_outlined),
  localModels('Local models', Icons.memory_outlined),
  agents('Agents', Icons.smart_toy_outlined),
  skills('Skills', Icons.auto_awesome_outlined),
  knowledge('Knowledge bases', Icons.storage_outlined),
  memory('Memories', Icons.psychology_outlined),
  connections('MCP & external access', Icons.cable_outlined),
  privacy('Privacy & data', Icons.shield_outlined),
  appearance('Appearance', Icons.palette_outlined),
  plugins('Plugins', Icons.extension_outlined),
  system('System information', Icons.info_outline);

  const _Section(this.label, this.icon);
  final String label;
  final IconData icon;
}

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({this.autoLoad = true, super.key});

  final bool autoLoad;

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  _Section section = _Section.providers;

  @override
  void initState() {
    super.initState();
    if (widget.autoLoad) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => ref.read(settingsProvider.notifier).load(),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(settingsProvider);
    final colors = shad.Theme.of(context).colorScheme;
    final brightness = colors.background.computeLuminance() < .5
        ? Brightness.dark
        : Brightness.light;
    final materialTheme = ThemeData(
      brightness: brightness,
      useMaterial3: true,
      colorScheme: ColorScheme(
        brightness: brightness,
        primary: colors.primary,
        onPrimary: colors.primaryForeground,
        secondary: colors.secondary,
        onSecondary: colors.secondaryForeground,
        error: colors.destructive,
        onError: brightness == Brightness.dark
            ? colors.background
            : Colors.white,
        surface: colors.background,
        onSurface: colors.foreground,
      ),
      canvasColor: colors.background,
      cardColor: colors.card,
      dividerColor: Colors.transparent,
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: colors.input,
        labelStyle: TextStyle(color: colors.mutedForeground),
        hintStyle: TextStyle(color: colors.mutedForeground),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(14),
          borderSide: BorderSide.none,
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(14),
          borderSide: BorderSide.none,
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(14),
          borderSide: BorderSide.none,
        ),
      ),
    );
    return KnowMeScreen(
      title: 'Settings',
      trailing: [
        shad.GhostButton(
          onPressed: context.pop,
          density: shad.ButtonDensity.icon,
          child: const Icon(Icons.close),
        ),
      ],
      child: Theme(
        data: materialTheme,
        child: Material(
          type: MaterialType.transparency,
          textStyle: TextStyle(color: colors.foreground),
          child: LayoutBuilder(
            builder: (context, constraints) {
              final wide = constraints.maxWidth >= 820;
              final navigation = _SettingsNavigation(
                section: section,
                horizontal: !wide,
                onSelect: (value) => setState(() => section = value),
              );
              final content = _SectionContent(section: section, state: state);
              return Column(
                children: [
                  if (state.loading)
                    const LinearProgressIndicator(minHeight: 2),
                  if (state.error != null || state.notice != null)
                    Semantics(
                      liveRegion: true,
                      child: Container(
                        width: double.infinity,
                        color: state.error == null
                            ? colors.accent
                            : colors.destructive.withValues(alpha: .14),
                        padding: const EdgeInsets.all(12),
                        child: Text(
                          state.error ?? state.notice!,
                          style: TextStyle(
                            color: state.error == null
                                ? colors.accentForeground
                                : colors.destructive,
                          ),
                        ),
                      ),
                    ),
                  Expanded(
                    child: wide
                        ? Row(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              SizedBox(width: 244, child: navigation),
                              Expanded(child: content),
                            ],
                          )
                        : Column(
                            children: [
                              SizedBox(height: 64, child: navigation),
                              Expanded(child: content),
                            ],
                          ),
                  ),
                ],
              );
            },
          ),
        ),
      ),
    );
  }
}

class _SettingsNavigation extends StatelessWidget {
  const _SettingsNavigation({
    required this.section,
    required this.horizontal,
    required this.onSelect,
  });
  final _Section section;
  final bool horizontal;
  final ValueChanged<_Section> onSelect;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final children = [
      for (final item in _Section.values)
        Padding(
          padding: const EdgeInsets.all(4),
          child: Semantics(
            selected: item == section,
            button: true,
            child: InkWell(
              borderRadius: BorderRadius.circular(14),
              onTap: () => onSelect(item),
              child: Container(
                constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(
                  color: item == section ? colors.card : null,
                  borderRadius: BorderRadius.circular(14),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(item.icon, size: 19),
                    const SizedBox(width: 10),
                    Text(
                      item.label,
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
    ];
    return ColoredBox(
      color: colors.secondary,
      child: horizontal
          ? ListView(scrollDirection: Axis.horizontal, children: children)
          : ListView(padding: const EdgeInsets.all(8), children: children),
    );
  }
}

class _SectionContent extends ConsumerWidget {
  const _SectionContent({required this.section, required this.state});
  final _Section section;
  final SettingsState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notifier = ref.read(settingsProvider.notifier);
    final child = switch (section) {
      _Section.providers => _ProvidersPanel(state: state),
      _Section.runtime => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _RuntimePanel(state: state),
          const SizedBox(height: 28),
          _GlobalPolicyPanel(state: state),
        ],
      ),
      _Section.localModels => _LocalModelsPanel(state: state),
      _Section.agents => _AgentsPanel(state: state),
      _Section.skills => _SkillsPanel(state: state),
      _Section.knowledge => _KnowledgePanel(state: state),
      _Section.memory => _MemoryPanel(state: state),
      _Section.connections => _McpPanel(state: state),
      _Section.privacy => _RegistryPanel(
        title: 'Privacy & data',
        caption: 'Device registration, peer discovery, CRDT/gossip keys, sync and data governance are persisted by UAR.',
        settings: _settings(state.snapshot.admin).where((item) {
          final key = item['key']?.toString() ?? '';
          return key.startsWith('privacy.') ||
              key.startsWith('sync.') ||
              key.startsWith('device.');
        }).toList(),
      ),
      _Section.appearance => const _AppearancePanel(),
      _Section.plugins => _ResourcePanel(
        title: 'Plugins',
        caption: 'WASM plugin schemas are registered in UAR; operational plugin interfaces render through A2UI.',
        values: _objects(state.snapshot.admin['settingTypes']),
        icon: Icons.extension_outlined,
      ),
      _Section.system => _RegistryPanel(
        title: 'System & inference',
        caption: 'Runtime ports, context policy, skill strategy and machine-facing settings supplied by UAR.',
        settings: _settings(state.snapshot.admin),
      ),
    };
    return RefreshIndicator(
      onRefresh: notifier.load,
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 920),
            child: child,
          ),
        ],
      ),
    );
  }
}

class _PanelHeader extends StatelessWidget {
  const _PanelHeader({
    required this.title,
    required this.caption,
    required this.icon,
  });
  final String title;
  final String caption;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 44,
          height: 44,
          decoration: BoxDecoration(
            color: colors.card,
            borderRadius: BorderRadius.circular(14),
          ),
          child: Icon(icon, color: colors.primary),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: const TextStyle(
                  fontSize: 24,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                caption,
                style: TextStyle(color: colors.mutedForeground, height: 1.4),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// On mobile the Universal Agent Runtime is ALWAYS the embedded in-process
/// kernel — there is no service URL, API key, or session to configure. This
/// panel is a read-only status surface; the model the runtime uses is chosen in
/// the "Model providers" tab (on-device or a configured cloud provider).
class _RuntimePanel extends ConsumerWidget {
  const _RuntimePanel({required this.state});
  final SettingsState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    final cloudModel = state.snapshot.cloudModel;
    final activeLabel = switch (cloudModel) {
      {'providerId': final String provider, 'modelId': final String model}
          when provider.isNotEmpty && model.isNotEmpty =>
        '$provider / $model',
      _ => 'On-device model',
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Runtime & policy',
          caption:
              'KnowMe runs the Universal Agent Runtime in-process on this '
              'device — agents, skills, tools, memory, knowledge, A2UI and '
              'policy. Choose which model it uses under Model providers.',
          icon: Icons.hub_outlined,
        ),
        const SizedBox(height: 20),
        _Card(
          child: Row(
            children: [
              Icon(Icons.memory_outlined, color: colors.primary),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Embedded · on-device',
                      style: TextStyle(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      'Active model: $activeLabel',
                      style: TextStyle(color: colors.onSurfaceVariant),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              _StatusPill(label: 'Active', color: colors.primary),
            ],
          ),
        ),
      ],
    );
  }
}

/// A small text-first status pill. Status is conveyed by the label text (not
/// color alone) to satisfy the WCAG "status not by color alone" rule.
class _StatusPill extends StatelessWidget {
  const _StatusPill({required this.label, required this.color});
  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.14),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.check_circle_outline, size: 14, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              color: color,
              fontWeight: FontWeight.w600,
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }
}

class _GlobalPolicyPanel extends ConsumerWidget {
  const _GlobalPolicyPanel({required this.state});

  final SettingsState state;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = _settings(state.snapshot.admin);
    final setting = settings.cast<Map<String, dynamic>?>().firstWhere(
      (item) => item?['key'] == 'run_policy.global',
      orElse: () => null,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Global run policy',
          caption: 'The UAR resolves these defaults with each agent and conversation override before it assembles context or exposes tools.',
          icon: Icons.account_tree_outlined,
        ),
        const SizedBox(height: 20),
        if (setting == null)
          const _Card(
            child: Text(
              // There is no service to restart: on mobile the UAR is ALWAYS
              // the embedded in-process runtime (see the note above). Telling
              // the user to restart one sends them looking for something that
              // does not exist on this platform.
              'The embedded runtime has not registered run_policy.global yet. '
              'Pull to refresh; if it persists, restart the app.',
            ),
          )
        else ...[
          _Card(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Default model',
                  style: TextStyle(fontWeight: FontWeight.w700),
                ),
                const SizedBox(height: 4),
                Text(
                  'The model every conversation uses unless an agent or '
                  'conversation overrides it. Leave on runtime default to use '
                  'the on-device model.',
                  style: TextStyle(
                    fontSize: 12,
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
                const SizedBox(height: 12),
                ModelRoutePicker(
                  providers: state.snapshot.providers,
                  value: _routeFromPolicy(setting['data']),
                  inheritLabel: 'Runtime default',
                  enabled: !state.saving,
                  onChanged: (route) => ref
                      .read(settingsProvider.notifier)
                      .saveSetting(
                        'run_policy.global',
                        _policyWithModel(setting['data'], route),
                      ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 12),
          _SettingEditor(
            setting: setting,
            save: (value) => ref
                .read(settingsProvider.notifier)
                .saveSetting('run_policy.global', value),
          ),
        ],
      ],
    );
  }
}

/// Read a `{ providerId, modelId }` route from a policy value's `model` field.
Map<String, String>? _routeFromPolicy(dynamic policy) {
  if (policy is Map) {
    final model = policy['model'];
    if (model is Map) {
      final provider = model['provider_id']?.toString() ?? '';
      final id = model['model_id']?.toString() ?? '';
      if (provider.isNotEmpty && id.isNotEmpty) {
        return {'providerId': provider, 'modelId': id};
      }
    }
  }
  return null;
}

/// Return a copy of the policy object with its `model` field set from a route
/// (or cleared to null when the route is null).
Map<String, dynamic> _policyWithModel(
  dynamic policy,
  Map<String, String>? route,
) {
  final next = <String, dynamic>{
    if (policy is Map) ...policy.cast<String, dynamic>(),
  };
  next['model'] = route == null
      ? null
      : {'provider_id': route['providerId'], 'model_id': route['modelId']};
  return next;
}

class _LocalModelsPanel extends ConsumerWidget {
  const _LocalModelsPanel({required this.state});
  final SettingsState state;
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final model = state.snapshot.localModel;
    final diagnostics = state.snapshot.localDiagnostics;
    final busy = const {
      'downloading',
      'verifying',
      'loading',
      'cancelling',
    }.contains(model['state']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Local models',
          caption: 'The KnowMe container owns local models and exposes them to UAR through the local provider contract.',
          icon: Icons.memory_outlined,
        ),
        const SizedBox(height: 20),
        _Card(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                model['displayName']?.toString() ??
                    model['modelId']?.toString() ??
                    'Android Gemma 4 E2B',
                style: const TextStyle(
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 6),
              Text(
                'State: ${model['state'] ?? 'notInstalled'} · Backend: ${diagnostics['backend'] ?? diagnostics['activeBackend'] ?? 'not loaded'}',
              ),
              // Rust computes a fallback reason whenever the engine runs on
              // something other than its first choice — CPU instead of GPU/NPU,
              // or a smaller model after a memory preflight refusal. It was
              // never rendered, so the one place a user (or a certification
              // run) could see WHY the device is on this backend showed only
              // the backend itself.
              if (diagnostics['fallback_reason'] != null ||
                  diagnostics['fallbackReason'] != null) ...[
                const SizedBox(height: 4),
                Semantics(
                  liveRegion: true,
                  child: Text(
                    'Fallback: ${diagnostics['fallback_reason'] ?? diagnostics['fallbackReason']}',
                    style: TextStyle(
                      color: shad.Theme.of(context).colorScheme.mutedForeground,
                    ),
                  ),
                ),
              ],
              if (model['progress'] is num) ...[
                const SizedBox(height: 12),
                LinearProgressIndicator(
                  value: (model['progress'] as num).toDouble().clamp(0, 1),
                ),
              ],
              const SizedBox(height: 16),
              Wrap(
                spacing: 8,
                children: [
                  shad.PrimaryButton(
                    onPressed: state.saving || busy
                        ? null
                        : ref.read(settingsProvider.notifier).prepareLocalModel,
                    child: const Text('Prepare model'),
                  ),
                  if (busy)
                    shad.SecondaryButton(
                      onPressed: ref
                          .read(settingsProvider.notifier)
                          .cancelModelOperation,
                      child: const Text('Cancel'),
                    ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        _LocalSystemPromptCard(state: state),
      ],
    );
  }
}

/// Editor for the local lane's system prompt.
///
/// Stateful because the field owns a controller: rebuilding from `state` on
/// every keystroke would fight the user's cursor. The controller is re-synced
/// only when the saved value actually changes (a save or a reset), so in-progress
/// edits survive unrelated Settings rebuilds.
class _LocalSystemPromptCard extends ConsumerStatefulWidget {
  const _LocalSystemPromptCard({required this.state});
  final SettingsState state;

  @override
  ConsumerState<_LocalSystemPromptCard> createState() =>
      _LocalSystemPromptCardState();
}

class _LocalSystemPromptCardState
    extends ConsumerState<_LocalSystemPromptCard> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.state.snapshot.localSystemPrompt,
  );

  @override
  void didUpdateWidget(_LocalSystemPromptCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    final saved = widget.state.snapshot.localSystemPrompt;
    if (saved != oldWidget.state.snapshot.localSystemPrompt &&
        saved != _controller.text) {
      _controller.text = saved;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _reset() async {
    final notifier = ref.read(settingsProvider.notifier);
    // Ask Rust for the default rather than hardcoding the copy here, so the two
    // cannot drift.
    final shipped = await notifier.defaultLocalSystemPrompt();
    if (!mounted) return;
    _controller.text = shipped;
    await notifier.saveLocalSystemPrompt(shipped);
  }

  @override
  Widget build(BuildContext context) {
    final notifier = ref.read(settingsProvider.notifier);
    final saving = widget.state.saving;
    return _Card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Local system prompt',
            style: TextStyle(fontSize: 18, fontWeight: FontWeight.w700),
          ),
          const SizedBox(height: 6),
          const Text(
            'Sent as the first message on every local run. Leave it blank and '
            'save to restore the shipped default.',
          ),
          const SizedBox(height: 12),
          // The visible heading above is not programmatically tied to the field,
          // and a placeholder vanishes once text is entered — so the accessible
          // name lives here rather than relying on either.
          Semantics(
            textField: true,
            multiline: true,
            label: 'Local system prompt',
            hint:
                'Sent as the first message on every local run. '
                'Leave blank and save to restore the default.',
            child: shad.TextArea(
              controller: _controller,
              initialHeight: 140,
              expandableHeight: true,
              placeholder: const Text('System prompt for the on-device model'),
            ),
          ),
          const SizedBox(height: 16),
          // Wrap, so the two actions reflow instead of overflowing at large
          // text scales rather than clipping.
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              Semantics(
                button: true,
                enabled: !saving,
                label: 'Save local system prompt',
                // Excluded so the row announces one name, not "Save local
                // system prompt, Save prompt".
                excludeSemantics: true,
                child: shad.PrimaryButton(
                  onPressed: saving
                      ? null
                      : () => notifier.saveLocalSystemPrompt(_controller.text),
                  child: const Text('Save prompt'),
                ),
              ),
              Semantics(
                button: true,
                enabled: !saving,
                label: 'Reset local system prompt to the default',
                excludeSemantics: true,
                child: shad.SecondaryButton(
                  onPressed: saving ? null : _reset,
                  child: const Text('Reset to default'),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ProvidersPanel extends ConsumerStatefulWidget {
  const _ProvidersPanel({required this.state});
  final SettingsState state;
  @override
  ConsumerState<_ProvidersPanel> createState() => _ProvidersPanelState();
}

class _ProvidersPanelState extends ConsumerState<_ProvidersPanel> {
  String? selectedId;
  final search = TextEditingController();
  final apiKey = TextEditingController();
  final baseUrl = TextEditingController();
  String? defaultModel;
  bool enabled = true;
  Set<String> enabledModels = {};
  @override
  void dispose() {
    search.dispose();
    apiKey.dispose();
    baseUrl.dispose();
    super.dispose();
  }

  /// Trailing marker for a provider row. Prefers the ACTIVE marker, then the
  /// ON-DEVICE tag, then the configured "ON" badge. Text-based so status is
  /// never conveyed by color alone.
  Widget? _providerListMarker(
    BuildContext context, {
    required bool isActive,
    required bool isLocal,
    required bool isConfigured,
  }) {
    final colors = Theme.of(context).colorScheme;
    if (isActive) {
      return _StatusPill(label: 'Active', color: colors.primary);
    }
    if (isLocal) {
      return Text(
        'ON-DEVICE',
        style: TextStyle(
          color: colors.onSurfaceVariant,
          fontWeight: FontWeight.w700,
          fontSize: 11,
        ),
      );
    }
    if (isConfigured) {
      return Text(
        'ON',
        style: TextStyle(color: colors.primary, fontWeight: FontWeight.w700),
      );
    }
    return null;
  }

  void _select(Map<String, dynamic> provider) {
    final configured = widget.state.snapshot.configuredProviders
        .where((item) => item['id'] == provider['id'])
        .firstOrNull;
    setState(() {
      selectedId = provider['id']?.toString();
      baseUrl.text =
          configured?['baseUrl']?.toString() ??
          provider['defaultBaseUrl']?.toString() ??
          '';
      enabled = configured?['enabled'] != false;
      defaultModel =
          configured?['defaultModel']?.toString() ??
          widget.state.snapshot.cloudModel?['modelId']?.toString();
      final models = _objects(configured?['models'] ?? provider['models']);
      enabledModels = models
          .where((model) => model['enabled'] != false)
          .map((model) => model['id'].toString())
          .toSet();
      apiKey.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    final providers =
        widget.state.snapshot.providers.where((provider) {
            final q = search.text.trim().toLowerCase();
            return q.isEmpty ||
                '${provider['displayName']} ${provider['id']}'
                    .toLowerCase()
                    .contains(q);
          }).toList()
          // The on-device model is pinned first so "both" reads as one unified
          // picker; the rest keep the catalog order (configured-first from Rust).
          ..sort((a, b) {
            final aLocal = a['id'] == _localProviderId ? 0 : 1;
            final bLocal = b['id'] == _localProviderId ? 0 : 1;
            return aLocal.compareTo(bLocal);
          });
    final activeProviderId = widget.state.snapshot.cloudModel?['providerId']
        ?.toString();
    final selected =
        providers.where((item) => item['id'] == selectedId).firstOrNull ??
        providers.firstOrNull;
    if (selected != null && selectedId == null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && selectedId == null) _select(selected);
      });
    }
    final models = _objects(selected?['models']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Model providers',
          caption:
              'Choose the model the on-device runtime uses: the built-in '
              'on-device model, or a cloud provider you configure with an API '
              'key. Keys are stored securely on this device.',
          icon: Icons.cloud_outlined,
        ),
        const SizedBox(height: 20),
        if (providers.isEmpty)
          _Card(
            child: const Text(
              'Loading models from the on-device runtime… The on-device model '
              'works offline with no key; add a cloud provider below to use a '
              'hosted model.',
            ),
          )
        else
          LayoutBuilder(
            builder: (context, constraints) {
              final wide = constraints.maxWidth >= 680;
              final list = Column(
                children: [
                  TextField(
                    controller: search,
                    onChanged: (_) => setState(() {}),
                    decoration: const InputDecoration(
                      prefixIcon: Icon(Icons.search),
                      hintText: 'Search providers',
                    ),
                  ),
                  const SizedBox(height: 8),
                  for (final provider in providers)
                    ListTile(
                      selected: provider['id'] == selected?['id'],
                      leading: provider['id'] == _localProviderId
                          ? const CircleAvatar(
                              child: Icon(Icons.memory_outlined, size: 20),
                            )
                          : CircleAvatar(
                              child: Text(
                                (provider['displayName']?.toString() ??
                                        provider['id'].toString())
                                    .substring(0, 1)
                                    .toUpperCase(),
                              ),
                            ),
                      title: Text(
                        provider['displayName']?.toString() ??
                            provider['id'].toString(),
                      ),
                      trailing: _providerListMarker(
                        context,
                        isActive: provider['id'] == activeProviderId,
                        isLocal: provider['id'] == _localProviderId,
                        isConfigured: provider['configured'] == true,
                      ),
                      onTap: () => _select(provider),
                    ),
                ],
              );
              final isLocal = selected?['id'] == _localProviderId;
              final detail = selected == null
                  ? const SizedBox.shrink()
                  : _Card(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Row(
                            children: [
                              Expanded(
                                child: Text(
                                  selected['displayName']?.toString() ??
                                      selected['id'].toString(),
                                  style: const TextStyle(
                                    fontSize: 20,
                                    fontWeight: FontWeight.w800,
                                  ),
                                ),
                              ),
                              if (selected['id'] == activeProviderId ||
                                  (isLocal && activeProviderId == null))
                                _StatusPill(
                                  label: 'Active',
                                  color: Theme.of(context).colorScheme.primary,
                                ),
                              // The on-device provider is always enabled and has
                              // no credential to toggle.
                              if (!isLocal)
                                Switch(
                                  value: enabled,
                                  onChanged: (value) =>
                                      setState(() => enabled = value),
                                ),
                            ],
                          ),
                          // The on-device model needs no API key or host.
                          if (!isLocal) ...[
                            TextField(
                              controller: apiKey,
                              obscureText: true,
                              decoration: const InputDecoration(
                                labelText: 'API key',
                                hintText: 'Stored securely on this device',
                              ),
                            ),
                            const SizedBox(height: 10),
                            TextField(
                              controller: baseUrl,
                              decoration: const InputDecoration(
                                labelText: 'API host · optional',
                              ),
                            ),
                          ],
                          const SizedBox(height: 16),
                          const Text(
                            'Models',
                            style: TextStyle(fontWeight: FontWeight.w700),
                          ),
                          for (final model in models)
                            Material(
                              type: MaterialType.transparency,
                              child: CheckboxListTile(
                                contentPadding: EdgeInsets.zero,
                                value: enabledModels.contains(
                                  model['id'].toString(),
                                ),
                                title: Text(
                                  model['displayName']?.toString() ??
                                      model['id'].toString(),
                                ),
                                subtitle: Text(
                                  [
                                    if (model['contextWindow'] != null)
                                      '${model['contextWindow']} context',
                                    if (model['supportsTools'] == true) 'tools',
                                    if (model['supportsReasoning'] == true)
                                      'reasoning',
                                    if (model['supportsVision'] == true)
                                      'vision',
                                  ].join(' · '),
                                ),
                                secondary: IconButton(
                                  tooltip: 'Set as default model',
                                  onPressed: () => setState(
                                    () => defaultModel = model['id'].toString(),
                                  ),
                                  icon: Icon(
                                    defaultModel == model['id'].toString()
                                        ? Icons.star
                                        : Icons.star_border,
                                  ),
                                ),
                                onChanged: (value) => setState(
                                  () => value == true
                                      ? enabledModels.add(
                                          model['id'].toString(),
                                        )
                                      : enabledModels.remove(
                                          model['id'].toString(),
                                        ),
                                ),
                              ),
                            ),
                          const SizedBox(height: 12),
                          Wrap(
                            spacing: 8,
                            runSpacing: 8,
                            children: [
                              // "Use this model" makes the selected provider +
                              // its default model the active route for the
                              // embedded runtime. Available for both local and
                              // cloud (one unified selection).
                              shad.PrimaryButton(
                                onPressed:
                                    widget.state.saving ||
                                        (!isLocal && defaultModel == null)
                                    ? null
                                    : () => ref
                                          .read(settingsProvider.notifier)
                                          .setActiveRoute(
                                            selected['id'].toString(),
                                            isLocal
                                                ? (defaultModel ??
                                                      (models.isNotEmpty
                                                          ? models.first['id']
                                                                .toString()
                                                          : ''))
                                                : defaultModel!,
                                          ),
                                child: const Text('Use this model'),
                              ),
                              // Local has no credential/host to save, no remote
                              // to test, and cannot be removed.
                              if (!isLocal) ...[
                                shad.SecondaryButton(
                                  onPressed: widget.state.saving
                                      ? null
                                      : () {
                                          final modelValues = models
                                              .map(
                                                (model) => {
                                                  ...model,
                                                  'enabled': enabledModels
                                                      .contains(
                                                        model['id'].toString(),
                                                      ),
                                                },
                                              )
                                              .toList();
                                          ref
                                              .read(settingsProvider.notifier)
                                              .saveProvider({
                                                'id': selected['id'],
                                                'kind': selected['id'],
                                                'displayName':
                                                    selected['displayName'],
                                                'baseUrl':
                                                    baseUrl.text.trim().isEmpty
                                                    ? null
                                                    : baseUrl.text.trim(),
                                                'apiKey':
                                                    apiKey.text.trim().isEmpty
                                                    ? null
                                                    : apiKey.text.trim(),
                                                'enabled': enabled,
                                                'defaultModel': defaultModel,
                                                'models': modelValues,
                                              });
                                        },
                                  child: const Text('Save provider'),
                                ),
                                shad.SecondaryButton(
                                  onPressed: widget.state.saving
                                      ? null
                                      : () => ref
                                            .read(settingsProvider.notifier)
                                            .testProvider(
                                              selected['id'].toString(),
                                              defaultModel,
                                            ),
                                  child: const Text('Test connection'),
                                ),
                                if (selected['configured'] == true)
                                  shad.GhostButton(
                                    onPressed: () => ref
                                        .read(settingsProvider.notifier)
                                        .deleteProvider(
                                          selected['id'].toString(),
                                        ),
                                    child: const Text('Remove'),
                                  ),
                              ],
                            ],
                          ),
                        ],
                      ),
                    );
              return wide
                  ? Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        SizedBox(width: 270, child: list),
                        const SizedBox(width: 14),
                        Expanded(child: detail),
                      ],
                    )
                  : Column(
                      children: [
                        ConstrainedBox(
                          constraints: const BoxConstraints(maxHeight: 320),
                          child: SingleChildScrollView(child: list),
                        ),
                        const SizedBox(height: 14),
                        detail,
                      ],
                    );
            },
          ),
      ],
    );
  }
}

class _SkillsPanel extends ConsumerWidget {
  const _SkillsPanel({required this.state});
  final SettingsState state;
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final skills = _objects(state.snapshot.admin['skills']);
    final configured = {
      for (final item in _settings(state.snapshot.admin))
        item['key']?.toString(): item,
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Skills',
          caption: 'UAR discovers, selects, activates and persists skills. These global switches feed the same resolver as agent and conversation policy.',
          icon: Icons.auto_awesome_outlined,
        ),
        const SizedBox(height: 20),
        for (final skill in skills)
          Builder(
            builder: (_) {
              final id =
                  skill['skill_id']?.toString() ??
                  skill['id']?.toString() ??
                  '';
              final short = id.split('::').last;
              final key = 'skill_config.$short';
              final current = _map(configured[key]?['data']);
              final enabled = current['enabled'] != false;
              return _ToggleCard(
                title: skill['title']?.toString() ?? short,
                caption: skill['description']?.toString() ?? '',
                value: enabled,
                onChanged: (value) =>
                    ref.read(settingsProvider.notifier).saveSetting(key, {
                      ...current,
                      'enabled': value,
                      'description': skill['description'],
                    }),
              );
            },
          ),
      ],
    );
  }
}

class _KnowledgePanel extends ConsumerStatefulWidget {
  const _KnowledgePanel({required this.state});
  final SettingsState state;
  @override
  ConsumerState<_KnowledgePanel> createState() => _KnowledgePanelState();
}

class _KnowledgePanelState extends ConsumerState<_KnowledgePanel> {
  final name = TextEditingController();
  final description = TextEditingController();
  @override
  void dispose() {
    name.dispose();
    description.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final bases = _objects(widget.state.snapshot.admin['knowledgeBases']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Knowledge bases',
          caption: 'UAR owns indexing, retrieval, citations and provenance. Conversation policy can inherit, select or deny any base.',
          icon: Icons.storage_outlined,
        ),
        const SizedBox(height: 20),
        _Card(
          child: Column(
            children: [
              TextField(
                controller: name,
                decoration: const InputDecoration(
                  labelText: 'Knowledge base name',
                ),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: description,
                decoration: const InputDecoration(labelText: 'Description'),
              ),
              const SizedBox(height: 12),
              Align(
                alignment: Alignment.centerRight,
                child: shad.PrimaryButton(
                  onPressed: name.text.trim().isEmpty || widget.state.saving
                      ? null
                      : () => ref
                            .read(settingsProvider.notifier)
                            .createKnowledgeBase(
                              name.text.trim(),
                              description.text,
                            ),
                  child: const Text('Create'),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        for (final base in bases)
          _ResourceTile(
            value: base,
            title: base['name']?.toString() ?? base['id'].toString(),
            subtitle: base['description']?.toString() ?? 'No description',
            onDelete: base['name'] == 'default'
                ? null
                : () => ref
                      .read(settingsProvider.notifier)
                      .deleteKnowledgeBase(base['id'].toString()),
          ),
      ],
    );
  }
}

class _MemoryPanel extends ConsumerWidget {
  const _MemoryPanel({required this.state});
  final SettingsState state;
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final memoriesRoot = _map(state.snapshot.admin['memories']);
    final memories = _objects(memoriesRoot['items']);
    final setting = _settings(state.snapshot.admin)
        .where((item) => item['key'] == 'memory.enabled')
        .firstOrNull;
    final enabled = setting?['data'] != false;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Memories',
          caption: 'Governed memories live in UAR storage. Model reasoning is never promoted to future context by default.',
          icon: Icons.psychology_outlined,
        ),
        const SizedBox(height: 20),
        _ToggleCard(
          title: 'Memory retrieval',
          caption: 'Apply governed memories during context assembly.',
          value: enabled,
          onChanged: (value) => ref
              .read(settingsProvider.notifier)
              .saveSetting('memory.enabled', value),
        ),
        const SizedBox(height: 12),
        for (final memory in memories)
          _ResourceTile(
            value: memory,
            title:
                memory['content']?.toString() ??
                memory['text']?.toString() ??
                'Memory',
            subtitle: memory['scope']?.toString() ?? '',
            onDelete: () => ref
                .read(settingsProvider.notifier)
                .deleteMemory(memory['id'].toString()),
          ),
      ],
    );
  }
}

class _McpPanel extends ConsumerStatefulWidget {
  const _McpPanel({required this.state});
  final SettingsState state;
  @override
  ConsumerState<_McpPanel> createState() => _McpPanelState();
}

class _McpPanelState extends ConsumerState<_McpPanel> {
  final name = TextEditingController();
  final endpoint = TextEditingController();
  @override
  void dispose() {
    name.dispose();
    endpoint.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final servers = _objects(
      _map(widget.state.snapshot.admin['mcpServers'])['servers'],
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'MCP & external access',
          caption: 'UAR owns MCP connections, tool discovery, execution and credentials. Chat receives only policy-approved tools.',
          icon: Icons.cable_outlined,
        ),
        const SizedBox(height: 20),
        _Card(
          child: Column(
            children: [
              TextField(
                controller: name,
                decoration: const InputDecoration(labelText: 'Server name'),
              ),
              const SizedBox(height: 10),
              TextField(
                controller: endpoint,
                decoration: const InputDecoration(
                  labelText: 'Remote HTTP endpoint',
                ),
              ),
              const SizedBox(height: 12),
              Align(
                alignment: Alignment.centerRight,
                child: shad.PrimaryButton(
                  onPressed: widget.state.saving
                      ? null
                      : () => ref.read(settingsProvider.notifier).saveMcpServer(
                          name.text.trim(),
                          {
                            'name': name.text.trim(),
                            'enabled': true,
                            'transport': 'remote_http',
                            'url': endpoint.text.trim(),
                            'env': <String, String>{},
                          },
                        ),
                  child: const Text('Add server'),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        for (final server in servers)
          _ResourceTile(
            value: server,
            title: server['name']?.toString() ?? 'MCP server',
            subtitle:
                '${server['transport'] ?? ''} · ${server['connected'] == true ? 'connected' : 'offline'}',
            onDelete: () => ref
                .read(settingsProvider.notifier)
                .deleteMcpServer(server['name'].toString()),
          ),
      ],
    );
  }
}

class _AppearancePanel extends ConsumerWidget {
  const _AppearancePanel();
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selected = ref.watch(themeModeProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Appearance',
          caption: 'Choose an accessible KnowMe palette or follow this device.',
          icon: Icons.palette_outlined,
        ),
        const SizedBox(height: 20),
        _Card(
          child: Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final option in const [
                (shad.ThemeMode.system, Icons.brightness_auto, 'System'),
                (shad.ThemeMode.light, Icons.light_mode_outlined, 'Light'),
                (shad.ThemeMode.dark, Icons.dark_mode_outlined, 'Dark'),
              ])
                option.$1 == selected
                    ? shad.PrimaryButton(
                        onPressed: () => ref
                            .read(themeModeProvider.notifier)
                            .select(option.$1),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(option.$2),
                            const SizedBox(width: 8),
                            Text(option.$3),
                          ],
                        ),
                      )
                    : shad.GhostButton(
                        onPressed: () => ref
                            .read(themeModeProvider.notifier)
                            .select(option.$1),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(option.$2),
                            const SizedBox(width: 8),
                            Text(option.$3),
                          ],
                        ),
                      ),
            ],
          ),
        ),
      ],
    );
  }
}

class _RegistryPanel extends ConsumerWidget {
  const _RegistryPanel({
    required this.title,
    required this.caption,
    required this.settings,
  });
  final String title;
  final String caption;
  final List<Map<String, dynamic>> settings;
  @override
  Widget build(BuildContext context, WidgetRef ref) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _PanelHeader(title: title, caption: caption, icon: Icons.tune_outlined),
      const SizedBox(height: 20),
      if (settings.isEmpty)
        const _Card(child: Text('No registered settings in this category yet.'))
      else
        for (final setting in settings.take(100))
          _SettingEditor(
            setting: setting,
            save: (value) => ref
                .read(settingsProvider.notifier)
                .saveSetting(setting['key'].toString(), value),
          ),
    ],
  );
}

class _SettingEditor extends StatefulWidget {
  const _SettingEditor({required this.setting, required this.save});
  final Map<String, dynamic> setting;
  final ValueChanged<Object?> save;
  @override
  State<_SettingEditor> createState() => _SettingEditorState();
}

class _SettingEditorState extends State<_SettingEditor> {
  late final TextEditingController controller = TextEditingController(
    text: widget.setting['data'] is String
        ? widget.setting['data'].toString()
        : jsonEncode(widget.setting['data']),
  );
  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final value = widget.setting['data'];
    if (value is bool) {
      return _ToggleCard(
        title:
            widget.setting['name']?.toString() ??
            widget.setting['key'].toString(),
        caption: widget.setting['key'].toString(),
        value: value,
        onChanged: widget.save,
      );
    }
    return _Card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            widget.setting['name']?.toString() ??
                widget.setting['key'].toString(),
            style: const TextStyle(fontWeight: FontWeight.w700),
          ),
          Text(
            widget.setting['key'].toString(),
            style: const TextStyle(fontSize: 11),
          ),
          const SizedBox(height: 8),
          TextField(controller: controller),
          const SizedBox(height: 8),
          Align(
            alignment: Alignment.centerRight,
            child: shad.SecondaryButton(
              onPressed: () {
                Object? parsed = controller.text;
                try {
                  parsed = jsonDecode(controller.text);
                } catch (_) {}
                widget.save(parsed);
              },
              child: const Text('Save'),
            ),
          ),
        ],
      ),
    );
  }
}

class _ResourcePanel extends StatelessWidget {
  const _ResourcePanel({
    required this.title,
    required this.caption,
    required this.values,
    required this.icon,
  });
  final String title;
  final String caption;
  final List<Map<String, dynamic>> values;
  final IconData icon;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _PanelHeader(title: title, caption: caption, icon: icon),
      const SizedBox(height: 20),
      if (values.isEmpty)
        const _Card(child: Text('No resources advertised by UAR.'))
      else
        for (final value in values)
          _ResourceTile(
            value: value,
            title:
                value['title']?.toString() ??
                value['name']?.toString() ??
                value['id']?.toString() ??
                value['key']?.toString() ??
                'Resource',
            subtitle:
                value['description']?.toString() ??
                value['id']?.toString() ??
                '',
          ),
    ],
  );
}

class _AgentsPanel extends ConsumerStatefulWidget {
  const _AgentsPanel({required this.state});

  final SettingsState state;

  @override
  ConsumerState<_AgentsPanel> createState() => _AgentsPanelState();
}

class _AgentsPanelState extends ConsumerState<_AgentsPanel> {
  String? selectedId;
  bool creating = false;
  final title = TextEditingController();
  final description = TextEditingController();
  final systemPrompt = TextEditingController();
  final policy = TextEditingController();

  @override
  void dispose() {
    title.dispose();
    description.dispose();
    systemPrompt.dispose();
    policy.dispose();
    super.dispose();
  }

  Map<String, dynamic>? get selected {
    final agents = _agents(widget.state.snapshot.admin);
    if (agents.isEmpty) return null;
    final id = selectedId;
    return id == null
        ? agents.first
        : agents.cast<Map<String, dynamic>?>().firstWhere(
            (agent) => agent?['id']?.toString() == id,
            orElse: () => agents.first,
          );
  }

  Map<String, dynamic> _defaultPolicy() => {
    'version': 1,
    'chat_mode': null,
    'agent_id': null,
    'model': null,
    for (final key in ['skills', 'tools', 'mcp_servers', 'knowledge_bases'])
      key: {'mode': 'inherit', 'ids': <String>[], 'denied_ids': <String>[]},
    'memory_enabled': null,
    'context_strategy': null,
    'tool_approval': 'inherit',
  };

  void _load(Map<String, dynamic> agent) {
    final metadata = _map(agent['metadata']);
    final prompt = _map(agent['prompt']);
    final extensions = _map(agent['extensions']);
    final savedPolicy = _map(extensions['uar.run_policy']);
    setState(() {
      creating = false;
      selectedId = agent['id']?.toString();
      title.text = metadata['title']?.toString() ?? selectedId ?? '';
      description.text = metadata['description']?.toString() ?? '';
      systemPrompt.text = prompt['system']?.toString() ?? '';
      policy.text = const JsonEncoder.withIndent('  ')
          .convert(savedPolicy.isEmpty ? _defaultPolicy() : savedPolicy);
    });
  }

  void _startNew() {
    setState(() {
      creating = true;
      selectedId = null;
      title.clear();
      description.clear();
      systemPrompt.clear();
      policy.text = const JsonEncoder.withIndent('  ')
          .convert(_defaultPolicy());
    });
  }

  /// Decode the current policy JSON in the editor, or an empty map if invalid.
  Map<String, dynamic> _decodePolicy() {
    try {
      final decoded = jsonDecode(policy.text);
      if (decoded is Map<String, dynamic>) return decoded;
    } on FormatException {
      // Fall through to empty — the picker degrades to "Global default".
    }
    return {};
  }

  /// Write a `{ provider_id, model_id }` model (or null) into the editor JSON,
  /// keeping the rest of the policy intact so the raw editor stays authoritative.
  void _setPolicyModel(Map<String, String>? route) {
    final next = _policyWithModel(_decodePolicy(), route);
    setState(() {
      policy.text = const JsonEncoder.withIndent('  ').convert(next);
    });
  }

  Map<String, dynamic> _newAgent(Map<String, dynamic> runPolicy) => {
    'version': '1.0.0',
    'kind': 'agent',
    'id': '',
    'metadata': {
      'title': title.text.trim(),
      'description': description.text.trim(),
      'tags': <String>[],
      'author': null,
      'icon': null,
    },
    'runtime': {'entry': 'default', 'protocols': <String, dynamic>{}},
    'policy': {
      'provider': {
        'default': {'provider': '', 'model': ''},
        'fallbacks': <dynamic>[],
      },
      'tools': {
        'allow': ['*'],
        'deny': <String>[],
        'max_concurrent': 1,
        'execution_mode': 'direct',
      },
      'skills': {'prefer': <String>[], 'max_active': 3},
    },
    'schemas': {'inputs': null, 'outputs': null, 'state': null},
    'prompt': {'system': systemPrompt.text, 'instructions': <String>[]},
    'memory': {
      'conversation': {'enabled': true},
      'kb': {
        'enabled': true,
        'knowledge_bases': <String>[],
        'citation_required': false,
      },
    },
    'tools': {'bundles': <dynamic>[]},
    'ui': {
      'forms': {'enabled': false},
      'artifacts': {'enabled': true, 'preferred_types': <String>[]},
    },
    'extensions': {'uar.run_policy': runPolicy},
  };

  Future<void> _save() async {
    Object? decoded;
    try {
      decoded = jsonDecode(policy.text);
    } on FormatException catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Invalid agent policy: ${error.message}')),
        );
      }
      return;
    }
    if (decoded is! Map<String, dynamic>) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Agent policy must be a JSON object.')),
        );
      }
      return;
    }
    final current = selected;
    final agent = creating || current == null
        ? _newAgent(decoded)
        : {
            ...current,
            'metadata': {
              ..._map(current['metadata']),
              'title': title.text.trim(),
              'description': description.text.trim(),
            },
            'prompt': {..._map(current['prompt']), 'system': systemPrompt.text},
            'extensions': {
              ..._map(current['extensions']),
              'uar.run_policy': decoded,
            },
          };
    await ref.read(settingsProvider.notifier).saveAgent(agent);
    if (mounted) setState(() => creating = false);
  }

  @override
  Widget build(BuildContext context) {
    final agents = _agents(widget.state.snapshot.admin);
    final current = selected;
    if (!creating && current != null && title.text.isEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && title.text.isEmpty) _load(current);
      });
    }
    final protected =
        !creating &&
        {'default-agent', 'orchestrator-agent'}.contains(current?['id']);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _PanelHeader(
          title: 'Agents',
          caption: 'Definitions, prompts, and effective execution policy are stored and enforced by UAR. Default and Orchestrator agents are protected.',
          icon: Icons.smart_toy_outlined,
        ),
        const SizedBox(height: 20),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            FilledButton.icon(
              onPressed: _startNew,
              icon: const Icon(Icons.add),
              label: const Text('Add agent'),
            ),
            for (final agent in agents)
              ChoiceChip(
                label: Text(
                  _map(agent['metadata'])['title']?.toString() ??
                      agent['id']?.toString() ??
                      'Agent',
                ),
                selected: !creating && current?['id'] == agent['id'],
                onSelected: (_) => _load(agent),
              ),
          ],
        ),
        const SizedBox(height: 16),
        _Card(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TextField(
                controller: title,
                onChanged: (_) => setState(() {}),
                decoration: const InputDecoration(labelText: 'Agent name'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: description,
                decoration: const InputDecoration(labelText: 'Description'),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: systemPrompt,
                minLines: 4,
                maxLines: 10,
                decoration: const InputDecoration(labelText: 'System prompt'),
              ),
              const SizedBox(height: 12),
              const Text(
                'Model',
                style: TextStyle(fontWeight: FontWeight.w700),
              ),
              const SizedBox(height: 4),
              Text(
                'Overrides the global default for this agent. Leave on global '
                'default to inherit.',
                style: TextStyle(
                  fontSize: 12,
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
              const SizedBox(height: 8),
              ModelRoutePicker(
                providers: widget.state.snapshot.providers,
                value: _routeFromPolicy(_decodePolicy()),
                inheritLabel: 'Global default',
                enabled: !widget.state.saving,
                onChanged: _setPolicyModel,
              ),
              const SizedBox(height: 12),
              TextField(
                controller: policy,
                minLines: 10,
                maxLines: 22,
                keyboardType: TextInputType.multiline,
                style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                decoration: const InputDecoration(
                  labelText: 'UAR run policy (JSON)',
                  helperText: 'Controls model, skills, tools, MCP servers, knowledge, memory, context, and approval at agent scope.',
                ),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  if (current != null && !protected && !creating)
                    TextButton.icon(
                      onPressed: widget.state.saving
                          ? null
                          : () => ref
                                .read(settingsProvider.notifier)
                                .deleteAgent(current['id'].toString()),
                      icon: const Icon(Icons.delete_outline),
                      label: const Text('Delete'),
                    ),
                  const Spacer(),
                  FilledButton.icon(
                    onPressed: widget.state.saving || title.text.trim().isEmpty
                        ? null
                        : _save,
                    icon: const Icon(Icons.save_outlined),
                    label: Text(widget.state.saving ? 'Saving…' : 'Save agent'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _ToggleCard extends StatelessWidget {
  const _ToggleCard({
    required this.title,
    required this.caption,
    required this.value,
    required this.onChanged,
  });
  final String title;
  final String caption;
  final bool value;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => _Card(
    child: Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: const TextStyle(fontWeight: FontWeight.w700)),
              if (caption.isNotEmpty)
                Text(caption, style: const TextStyle(fontSize: 13)),
            ],
          ),
        ),
        Switch(value: value, onChanged: onChanged),
      ],
    ),
  );
}

class _ResourceTile extends StatelessWidget {
  const _ResourceTile({
    required this.value,
    required this.title,
    required this.subtitle,
    this.onDelete,
  });
  final Map<String, dynamic> value;
  final String title;
  final String subtitle;
  final VoidCallback? onDelete;
  @override
  Widget build(BuildContext context) => _Card(
    child: Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontWeight: FontWeight.w700),
              ),
              if (subtitle.isNotEmpty)
                Text(
                  subtitle,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 13),
                ),
            ],
          ),
        ),
        if (onDelete != null)
          IconButton(
            onPressed: onDelete,
            icon: const Icon(Icons.delete_outline),
            tooltip: 'Delete',
          ),
      ],
    ),
  );
}

class _Card extends StatelessWidget {
  const _Card({required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Container(
      width: double.infinity,
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: colors.card,
        borderRadius: BorderRadius.circular(16),
      ),
      child: child,
    );
  }
}

Map<String, dynamic> _map(Object? value) => value is Map<String, dynamic>
    ? value
    : value is Map
    ? value.map((key, value) => MapEntry(key.toString(), value))
    : const {};
List<Map<String, dynamic>> _objects(Object? value) =>
    value is List ? value.map(_map).toList() : const [];
List<Map<String, dynamic>> _settings(Map<String, dynamic> admin) =>
    _objects(admin['settings']);
List<Map<String, dynamic>> _agents(Map<String, dynamic> admin) =>
    _objects(_map(admin['agents'])['runtime_agents']);
