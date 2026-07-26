// TJ-ARCH-MOB-001 compliant — conversation policy UI; UAR owns resolution.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import '../../../settings/presentation/providers/settings_notifier.dart';
import '../../../settings/presentation/widgets/model_route_picker.dart';
import '../providers/chat_control_notifier.dart';

class ChatPolicyToolbar extends ConsumerStatefulWidget {
  const ChatPolicyToolbar({
    required this.conversationId,
    this.autoLoad = true,
    super.key,
  });
  final String conversationId;
  final bool autoLoad;

  @override
  ConsumerState<ChatPolicyToolbar> createState() => _ChatPolicyToolbarState();
}

class _ChatPolicyToolbarState extends ConsumerState<ChatPolicyToolbar> {
  @override
  void initState() {
    super.initState();
    if (widget.autoLoad) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => ref
            .read(chatControlProvider(widget.conversationId).notifier)
            .load(),
      );
    }
  }

  @override
  void didUpdateWidget(covariant ChatPolicyToolbar oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.autoLoad && oldWidget.conversationId != widget.conversationId) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => ref
            .read(chatControlProvider(widget.conversationId).notifier)
            .load(),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(chatControlProvider(widget.conversationId));
    final notifier = ref.read(
      chatControlProvider(widget.conversationId).notifier,
    );
    final mode = state.policy['chat_mode']?.toString() ?? 'uar';
    final agents = _agents(state.catalog);
    final skills = _skills(state.catalog);
    final toolsRoot = _map(state.catalog['tools']);
    final tools = _objects(toolsRoot['tools'])
        .map(
          (value) => _Option(
            value['namespaced_name']?.toString() ??
                value['name']?.toString() ??
                '',
            value['name']?.toString() ??
                value['namespaced_name']?.toString() ??
                '',
          ),
        )
        .where((value) => value.id.isNotEmpty)
        .toList();
    final mcp =
        (toolsRoot['mcp_servers'] is List
                ? toolsRoot['mcp_servers'] as List
                : const [])
            .map((value) => _Option(value.toString(), value.toString()))
            .toList();
    // The catalog struct serializes camelCase (`knowledgeBases`); the policy
    // uses snake_case. Accept both so neither surface silently reads empty.
    final knowledge = _objects(
      state.catalog['knowledgeBases'] ?? state.catalog['knowledge_bases'],
    )
        .map(
          (value) => _Option(
            value['id']?.toString() ?? '',
            value['name']?.toString() ?? value['id']?.toString() ?? '',
          ),
        )
        .where((value) => value.id.isNotEmpty)
        .toList();
    return Semantics(
      label: 'Conversation agent and context controls',
      child: Material(
        color: shad.Theme.of(context).colorScheme.secondary,
        child: SizedBox(
          height: 58,
          child: ListView(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
            children: [
              // One control, labeled with the current choice. The list of modes
              // is "every defined agent + Local", which grows — Material 3
              // reserves segmented buttons for 2–5 fixed options and directs
              // longer lists to a menu, so this opens a selection sheet.
              _ModeButton(
                label: _modeLabel(mode, state.policy['agent_id'], agents),
                icon: _modeIcon(mode),
                selected: true,
                onTap: state.saving
                    ? null
                    : () => _chooseMode(context, notifier, state, agents),
              ),
              const SizedBox(width: 6),
              _ModeButton(
                label: 'Model ${_modelSummary(state.policy['model'])}',
                icon: Icons.developer_board_outlined,
                selected: _conversationRoute(state.policy['model']) != null,
                onTap: state.saving
                    ? null
                    : () => _chooseModel(context, notifier),
              ),
              _ResourceButton(
                label: 'Skills',
                icon: Icons.auto_awesome_outlined,
                selection: _map(state.policy['skills']),
                onTap: () => _chooseResources(
                  context,
                  'Skills',
                  'skills',
                  skills,
                  state,
                  notifier,
                ),
              ),
              _ResourceButton(
                label: 'Tools',
                icon: Icons.build_outlined,
                selection: _map(state.policy['tools']),
                onTap: () => _chooseResources(
                  context,
                  'Tools',
                  'tools',
                  tools,
                  state,
                  notifier,
                ),
              ),
              _ResourceButton(
                label: 'MCP',
                icon: Icons.cable_outlined,
                selection: _map(state.policy['mcp_servers']),
                onTap: () => _chooseResources(
                  context,
                  'MCP servers',
                  'mcp_servers',
                  mcp,
                  state,
                  notifier,
                ),
              ),
              _ResourceButton(
                label: 'Knowledge',
                icon: Icons.storage_outlined,
                selection: _map(state.policy['knowledge_bases']),
                onTap: () => _chooseResources(
                  context,
                  'Knowledge bases',
                  'knowledge_bases',
                  knowledge,
                  state,
                  notifier,
                ),
              ),
              _ModeButton(
                label: state.policy['memory_enabled'] == false
                    ? 'Memory off'
                    : 'Memory on',
                icon: Icons.psychology_outlined,
                selected: state.policy['memory_enabled'] != false,
                onTap: () => notifier.patch({
                  'memory_enabled': state.policy['memory_enabled'] == false,
                }),
              ),
              _ModeButton(
                label:
                    'Context ${_map(state.policy['context_strategy'])['type'] ?? 'inherit'}',
                icon: Icons.layers_outlined,
                selected: state.policy['context_strategy'] != null,
                onTap: () => _chooseContextStrategy(
                  context,
                  notifier,
                  _map(state.policy['context_strategy'])['type']?.toString() ??
                      'inherit',
                ),
              ),
              _ModeButton(
                label: 'Approval ${state.policy['tool_approval'] ?? 'inherit'}',
                icon: Icons.verified_user_outlined,
                selected:
                    (state.policy['tool_approval']?.toString() ?? 'inherit') !=
                    'inherit',
                onTap: () => _chooseToolApproval(
                  context,
                  notifier,
                  state.policy['tool_approval']?.toString() ?? 'inherit',
                ),
              ),
              if (state.saving)
                const Padding(
                  padding: EdgeInsets.all(12),
                  child: SizedBox.square(
                    dimension: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                ),
              if (state.error != null)
                Tooltip(
                  message: state.error!,
                  child: const Padding(
                    padding: EdgeInsets.all(10),
                    child: Icon(Icons.error_outline, color: Colors.red),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  /// Pick how this conversation runs: any defined agent (through the UAR), or
  /// the on-device model directly.
  ///
  /// Modes whose runtime is unavailable are shown disabled with the reason Rust
  /// reported, rather than offered as controls that fail when tapped.
  void _chooseMode(
    BuildContext context,
    ChatControl notifier,
    ChatControlState state,
    List<_Option> agents,
  ) {
    final mode = state.policy['chat_mode']?.toString() ?? 'uar';
    final agentId = state.policy['agent_id']?.toString();
    final localBlocked = state.laneBlocked('local');
    final uarBlocked = state.laneBlocked('uar');

    shad.openSheetOverlay<void>(
      context: context,
      position: shad.OverlayPosition.bottom,
      builder: (sheetContext) => SafeArea(
        child: ListView(
          shrinkWrap: true,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('Conversation mode').semiBold(),
                  const SizedBox(height: 4),
                  Text(
                    'Agents run through the Universal Agent Runtime. '
                    'Local talks to the on-device model directly.',
                    style: shad.Theme.of(context).typography.small.copyWith(
                      color: shad.Theme.of(
                        context,
                      ).colorScheme.mutedForeground,
                    ),
                  ),
                ],
              ),
            ),
            for (final agent in agents)
              _ModeChoiceTile(
                label: agent.label,
                // The agent lane is selected when this specific agent is the
                // conversation's agent — not merely when the mode is agentic.
                selected: mode != 'local' && agentId == agent.id,
                blockedReason: uarBlocked
                    ? state.laneAvailability['uar']
                    : null,
                onSelected: () {
                  shad.closeSheet(sheetContext);
                  notifier.setMode(
                    agent.id == 'orchestrator-agent' ? 'uar' : 'agent',
                    agentId: agent.id,
                  );
                },
              ),
            _ModeChoiceTile(
              label: 'Local',
              subtitle: 'On-device model, no agent runtime',
              selected: mode == 'local',
              blockedReason: localBlocked
                  ? state.laneAvailability['local']
                  : null,
              onSelected: () {
                shad.closeSheet(sheetContext);
                notifier.setMode('local');
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _chooseResources(
    BuildContext context,
    String title,
    String key,
    List<_Option> options,
    ChatControlState state,
    ChatControl notifier,
  ) async {
    final current = _map(state.policy[key]);
    var mode = current['mode']?.toString() ?? 'inherit';
    final ids = (current['ids'] is List ? current['ids'] as List : const [])
        .map((value) => value.toString())
        .toSet();
    final result = await showModalBottomSheet<(String, List<String>)>(
      context: context,
      isScrollControlled: true,
      builder: (context) => StatefulBuilder(
        builder: (context, setModalState) => SafeArea(
          child: SizedBox(
            height: MediaQuery.sizeOf(context).height * .70,
            child: Column(
              children: [
                ListTile(
                  title: Text(title),
                  subtitle: const Text(
                    'Conversation scope · UAR resolves global and agent limits.',
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  child: SegmentedButton<String>(
                    segments: const [
                      ButtonSegment(value: 'inherit', label: Text('Inherit')),
                      ButtonSegment(value: 'all', label: Text('All')),
                      ButtonSegment(value: 'none', label: Text('None')),
                      ButtonSegment(value: 'selected', label: Text('Select')),
                    ],
                    selected: {mode},
                    onSelectionChanged: (value) =>
                        setModalState(() => mode = value.first),
                  ),
                ),
                Expanded(
                  child: ListView(
                    children: [
                      for (final option in options)
                        CheckboxListTile(
                          value: mode == 'selected' && ids.contains(option.id),
                          title: Text(option.label),
                          onChanged: (checked) => setModalState(() {
                            mode = 'selected';
                            checked == true
                                ? ids.add(option.id)
                                : ids.remove(option.id);
                          }),
                        ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(12),
                  child: SizedBox(
                    width: double.infinity,
                    child: shad.PrimaryButton(
                      onPressed: () =>
                          Navigator.pop(context, (mode, ids.toList()..sort())),
                      child: const Text('Apply to conversation'),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
    if (result != null) await notifier.setSelection(key, result.$1, result.$2);
  }

  Future<void> _chooseContextStrategy(
    BuildContext context,
    ChatControl notifier,
    String current,
  ) async {
    final selected = await _chooseOne(context, 'Context strategy', const {
      'inherit': 'Inherit global and agent policy',
      'none': 'Full available history',
      'sliding_window': 'Keep the latest 20 messages',
      'summarize': 'Summarize older context',
      'truncate_middle': 'Keep first and latest messages',
      'hierarchical': 'Short, mid and long-term tiers',
    }, current);
    if (selected == null) return;
    final strategy = switch (selected) {
      'none' => <String, dynamic>{'type': 'none'},
      'sliding_window' => <String, dynamic>{
        'type': selected,
        'max_messages': 20,
      },
      'summarize' => <String, dynamic>{
        'type': selected,
        'threshold': 12,
        'summary_max_tokens': 500,
        'model': null,
      },
      'truncate_middle' => <String, dynamic>{
        'type': selected,
        'keep_first': 4,
        'keep_last': 16,
      },
      'hierarchical' => <String, dynamic>{
        'type': selected,
        'short_term_turns': 5,
        'mid_term_summary_tokens': 2000,
        'long_term_facts_tokens': 500,
      },
      _ => null,
    };
    await notifier.patch({'context_strategy': strategy});
  }

  Future<void> _chooseModel(BuildContext context, ChatControl notifier) async {
    // The provider catalog lives in the settings snapshot; the toolbar reads it
    // to offer the same providers/models as the settings model picker.
    final providers = ref.read(settingsProvider).snapshot.providers;
    final current = _conversationRoute(
      ref.read(chatControlProvider(widget.conversationId)).policy['model'],
    );
    await showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      builder: (sheetContext) => Padding(
        padding: EdgeInsets.only(
          left: 16,
          right: 16,
          top: 16,
          bottom: MediaQuery.of(sheetContext).viewInsets.bottom + 16,
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Conversation model',
              style: TextStyle(fontWeight: FontWeight.w700, fontSize: 16),
            ),
            const SizedBox(height: 4),
            Text(
              'Overrides the agent and global default for this chat only.',
              style: TextStyle(
                fontSize: 12,
                color: Theme.of(sheetContext).colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 16),
            ModelRoutePicker(
              providers: providers,
              value: current,
              inheritLabel: 'Agent / global default',
              onChanged: (route) {
                notifier.patch({
                  'model': route == null
                      ? null
                      : {
                          'provider_id': route['providerId'],
                          'model_id': route['modelId'],
                        },
                });
                Navigator.of(sheetContext).pop();
              },
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _chooseToolApproval(
    BuildContext context,
    ChatControl notifier,
    String current,
  ) async {
    final selected = await _chooseOne(context, 'Tool approval', const {
      'inherit': 'Inherit global and agent policy',
      'auto': 'Let UAR governance decide',
      'ask': 'Ask before mutating tool calls',
      'deny': 'Deny all tool execution',
    }, current);
    if (selected != null) await notifier.patch({'tool_approval': selected});
  }

  Future<String?> _chooseOne(
    BuildContext context,
    String title,
    Map<String, String> options,
    String current,
  ) => showModalBottomSheet<String>(
    context: context,
    builder: (context) => SafeArea(
      child: ListView(
        shrinkWrap: true,
        children: [
          ListTile(title: Text(title)),
          for (final entry in options.entries)
            ListTile(
              leading: Icon(
                entry.key == current
                    ? Icons.radio_button_checked
                    : Icons.radio_button_unchecked,
              ),
              title: Text(entry.key.replaceAll('_', ' ')),
              subtitle: Text(entry.value),
              onTap: () => Navigator.pop(context, entry.key),
            ),
        ],
      ),
    ),
  );
}

/// Label for the mode trigger: the resolved agent's name, or "Local".
///
/// Shows the agent by name rather than the generic word "Agent" — the point of
/// a single trigger is that it answers "what is this conversation running on?"
/// without opening it.
String _modeLabel(String mode, Object? agentId, List<_Option> agents) {
  if (mode == 'local') return 'Local';
  final id = agentId?.toString();
  final match = agents.where((agent) => agent.id == id).firstOrNull;
  if (match != null) return match.label;
  return id == null || id.isEmpty ? 'Orchestrator' : id;
}

IconData _modeIcon(String mode) =>
    mode == 'local' ? Icons.memory_outlined : Icons.hub_outlined;

/// One selectable mode in the picker sheet.
///
/// Selection is conveyed by a radio icon AND semantics (never colour alone),
/// and an unavailable mode is rendered disabled with its reason so the user
/// learns why instead of tapping a dead control.
class _ModeChoiceTile extends StatelessWidget {
  const _ModeChoiceTile({
    required this.label,
    required this.selected,
    required this.onSelected,
    this.subtitle,
    this.blockedReason,
  });

  final String label;
  final String? subtitle;
  final bool selected;
  final String? blockedReason;
  final VoidCallback onSelected;

  @override
  Widget build(BuildContext context) {
    final blocked = blockedReason != null;
    final detail = blocked ? blockedReason : subtitle;
    final theme = shad.Theme.of(context);
    // Blocked rows dim but keep their text legible — the reason is the point.
    final foreground = blocked
        ? theme.colorScheme.mutedForeground
        : theme.colorScheme.foreground;

    return Semantics(
      button: true,
      selected: selected,
      enabled: !blocked,
      label: label,
      hint: detail,
      // Excluded so the row reads as one control, not icon + two text nodes.
      excludeSemantics: true,
      child: InkWell(
        onTap: blocked ? null : onSelected,
        child: ConstrainedBox(
          // Project a11y floor: 48px minimum touch target.
          constraints: const BoxConstraints(minHeight: 48),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            child: Row(
              children: [
                Icon(
                  selected
                      ? Icons.radio_button_checked
                      : Icons.radio_button_unchecked,
                  size: 20,
                  color: foreground,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(label, style: TextStyle(color: foreground)),
                      if (detail != null)
                        Text(
                          detail,
                          style: theme.typography.small.copyWith(
                            color: theme.colorScheme.mutedForeground,
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _ModeButton extends StatelessWidget {
  const _ModeButton({
    required this.label,
    required this.icon,
    required this.selected,
    required this.onTap,
  });
  final String label;
  final IconData icon;
  final bool selected;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 2),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          constraints: const BoxConstraints(minWidth: 44, minHeight: 44),
          padding: const EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(
            color: selected ? colors.primary : colors.card,
            borderRadius: BorderRadius.circular(12),
          ),
          child: Row(
            children: [
              Icon(
                icon,
                size: 17,
                color: selected ? colors.primaryForeground : colors.foreground,
              ),
              const SizedBox(width: 6),
              Text(
                label,
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w700,
                  color: selected
                      ? colors.primaryForeground
                      : colors.foreground,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ResourceButton extends StatelessWidget {
  const _ResourceButton({
    required this.label,
    required this.icon,
    required this.selection,
    required this.onTap,
  });
  final String label;
  final IconData icon;
  final Map<String, dynamic> selection;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) {
    final mode = selection['mode']?.toString() ?? 'inherit';
    final count = selection['ids'] is List
        ? (selection['ids'] as List).length
        : 0;
    return _ModeButton(
      label: '$label ${mode == 'selected' ? count : mode}',
      icon: icon,
      selected: mode != 'inherit',
      onTap: onTap,
    );
  }
}

class _Option {
  const _Option(this.id, this.label);
  final String id;
  final String label;
}

Map<String, dynamic> _map(Object? value) => value is Map<String, dynamic>
    ? value
    : value is Map
    ? value.map((key, value) => MapEntry(key.toString(), value))
    : const {};
List<Map<String, dynamic>> _objects(Object? value) =>
    value is List ? value.map(_map).toList() : const [];
/// Read a `{ providerId, modelId }` route from a policy's `model` field.
Map<String, String>? _conversationRoute(dynamic model) {
  if (model is Map) {
    final provider = model['provider_id']?.toString() ?? '';
    final id = model['model_id']?.toString() ?? '';
    if (provider.isNotEmpty && id.isNotEmpty) {
      return {'providerId': provider, 'modelId': id};
    }
  }
  return null;
}

/// A compact chip label for the conversation model override.
String _modelSummary(dynamic model) {
  final route = _conversationRoute(model);
  if (route == null) return 'inherit';
  if (route['providerId'] == kOnDeviceProviderId) return 'On-device';
  return route['modelId'] ?? route['providerId'] ?? 'inherit';
}

List<_Option> _agents(Map<String, dynamic> catalog) =>
    _objects(_map(catalog['agents'])['runtime_agents'])
        .map(
          (value) => _Option(
            value['id']?.toString() ?? '',
            // An AgentArtifact carries its display name at `metadata.title`;
            // the flat `name`/`title` keys only appear on some service-shaped
            // payloads. Check metadata first so embedded surfaces show real
            // names instead of falling back to raw ids.
            _map(value['metadata'])['title']?.toString() ??
                value['name']?.toString() ??
                value['title']?.toString() ??
                value['id']?.toString() ??
                '',
          ),
        )
        .where((value) => value.id.isNotEmpty)
        .toList();
List<_Option> _skills(Map<String, dynamic> catalog) =>
    _objects(catalog['skills'])
        .map(
          (value) => _Option(
            value['skill_id']?.toString() ?? value['id']?.toString() ?? '',
            value['title']?.toString() ??
                value['name']?.toString() ??
                value['skill_id']?.toString() ??
                '',
          ),
        )
        .where((value) => value.id.isNotEmpty)
        .toList();
