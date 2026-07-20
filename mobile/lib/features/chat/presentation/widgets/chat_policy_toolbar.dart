// TJ-ARCH-MOB-001 compliant — conversation policy UI; UAR owns resolution.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

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
    final knowledge = _objects(state.catalog['knowledge_bases'])
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
              _ModeButton(
                label: 'Orchestrator',
                icon: Icons.hub_outlined,
                selected: mode == 'uar',
                onTap: state.saving ? null : () => notifier.setMode('uar'),
              ),
              _ModeButton(
                label: 'Agent',
                icon: Icons.smart_toy_outlined,
                selected: mode == 'agent',
                onTap: state.saving
                    ? null
                    : () => _chooseAgent(context, notifier, agents),
              ),
              _ModeButton(
                label: 'Local',
                icon: Icons.memory_outlined,
                selected: mode == 'local',
                onTap: state.saving ? null : () => notifier.setMode('local'),
              ),
              const SizedBox(width: 6),
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

  Future<void> _chooseAgent(
    BuildContext context,
    ChatControl notifier,
    List<_Option> agents,
  ) async {
    final selected = await showModalBottomSheet<String>(
      context: context,
      builder: (context) => SafeArea(
        child: ListView(
          shrinkWrap: true,
          children: [
            const ListTile(
              title: Text('Choose an agent'),
              subtitle: Text('Definitions and execution remain in UAR.'),
            ),
            for (final agent in agents)
              ListTile(
                title: Text(agent.label),
                onTap: () => Navigator.pop(context, agent.id),
              ),
          ],
        ),
      ),
    );
    if (selected != null) await notifier.setMode('agent', agentId: selected);
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
List<_Option> _agents(Map<String, dynamic> catalog) =>
    _objects(_map(catalog['agents'])['runtime_agents'])
        .map(
          (value) => _Option(
            value['id']?.toString() ?? '',
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
