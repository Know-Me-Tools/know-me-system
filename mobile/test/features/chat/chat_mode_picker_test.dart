// TJ-ARCH-MOB-001 compliant
//
// Contract tests for the conversation mode picker.
//
// The picker replaced a fixed Orchestrator/Agent/Local button row: the agent
// list is catalog-driven and grows, so a single trigger labelled with the
// current selection opens a sheet listing every agent plus Local. These tests
// pin the parts that broke in production — the agent artifact shape, the
// disabled-with-reason contract, and the rule that a rejected lane switch must
// never move the displayed selection.
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'package:knowme_poc/core/theme/tokens.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_control_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/widgets/chat_policy_toolbar.dart';

/// Agents as the runtime actually returns them: `AgentArtifact`s whose display
/// name lives at `metadata.title`, wrapped in the `{runtime_agents, ...}`
/// envelope the service contract defines. The embedded runtime once returned a
/// bare array here, which is why every embedded surface showed an empty list.
const _catalog = <String, dynamic>{
  'agents': {
    'runtime_agents': [
      {
        'id': 'orchestrator-agent',
        'metadata': {'title': 'Orchestrator'},
      },
      {
        'id': 'writer',
        'metadata': {'title': 'Writer'},
      },
    ],
    'federated_agents': <dynamic>[],
  },
  'skills': <dynamic>[],
  'tools': <String, dynamic>{},
  'knowledge_bases': <dynamic>[],
};

Map<String, dynamic> _policy({
  String chatMode = 'uar',
  String? agentId = 'orchestrator-agent',
}) => {
  'chat_mode': chatMode,
  'agent_id': agentId,
  'skills': {'mode': 'inherit', 'ids': <String>[]},
  'mcp_servers': {'mode': 'inherit', 'ids': <String>[]},
  'knowledge_bases': {'mode': 'inherit', 'ids': <String>[]},
};

Future<void> _pumpToolbar(WidgetTester tester, ChatControl control) =>
    tester.pumpWidget(
      ProviderScope(
        overrides: [
          chatControlProvider('conversation').overrideWith(() => control),
        ],
        child: shad.ShadcnApp(
          theme: KnowMeTheme.dark,
          // `openSheetOverlay` asserts on a DrawerOverlay ancestor. The running
          // app gets one from ShadcnApp.router's page scaffolding; the plain
          // `home:` constructor does not, so the test supplies it explicitly.
          home: const shad.DrawerOverlay(
            child: Scaffold(
              body: ChatPolicyToolbar(
                conversationId: 'conversation',
                autoLoad: false,
              ),
            ),
          ),
        ),
      ),
    );

void main() {
  testWidgets('mode trigger is labelled with the selected agent', (
    tester,
  ) async {
    await _pumpToolbar(
      tester,
      _FakeChatControl(
        const ChatControlState(catalog: _catalog).copyWith(
          policy: _policy(chatMode: 'agent', agentId: 'writer'),
        ),
      ),
    );

    // The agent's name, not the generic mode word — "Agent" tells the user
    // nothing about which agent is answering.
    expect(find.text('Writer'), findsOneWidget);
    expect(find.text('Agent'), findsNothing);
  });

  testWidgets('mode trigger is labelled Local when the local lane is chosen', (
    tester,
  ) async {
    await _pumpToolbar(
      tester,
      _FakeChatControl(
        const ChatControlState(catalog: _catalog)
            .copyWith(policy: _policy(chatMode: 'local', agentId: null)),
      ),
    );

    expect(find.text('Local'), findsOneWidget);
  });

  testWidgets('sheet lists every catalog agent plus Local', (tester) async {
    await _pumpToolbar(
      tester,
      _FakeChatControl(
        const ChatControlState(catalog: _catalog).copyWith(policy: _policy()),
      ),
    );

    await tester.tap(find.text('Orchestrator'));
    await tester.pumpAndSettle();

    expect(find.text('Conversation mode'), findsOneWidget);
    // Orchestrator appears twice once the sheet is open: the trigger label and
    // the sheet row. Writer and Local appear only in the sheet.
    expect(find.text('Orchestrator'), findsNWidgets(2));
    expect(find.text('Writer'), findsOneWidget);
    expect(find.text('Local'), findsOneWidget);
  });

  testWidgets('an unavailable lane renders disabled with its reason', (
    tester,
  ) async {
    const reason = 'No local model installed';
    final control = _FakeChatControl(
      const ChatControlState(
        catalog: _catalog,
        laneAvailability: {'local': reason},
      ).copyWith(policy: _policy()),
    );
    await _pumpToolbar(tester, control);

    await tester.tap(find.text('Orchestrator'));
    await tester.pumpAndSettle();

    // The reason is visible, not hidden behind a tooltip, and tapping the row
    // does not attempt a switch that Rust would reject.
    expect(find.text(reason), findsOneWidget);
    await tester.tap(find.text('Local'));
    await tester.pumpAndSettle();
    expect(control.requestedModes, isEmpty);
  });

  testWidgets('a rejected lane switch surfaces the error and keeps the '
      'displayed selection', (tester) async {
    final control = _RejectingChatControl(
      const ChatControlState(catalog: _catalog).copyWith(policy: _policy()),
    );
    await _pumpToolbar(tester, control);

    await tester.tap(find.text('Orchestrator'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Local'));
    await tester.pumpAndSettle();

    // Selection must not move optimistically: claiming Local while the switch
    // failed would route the next send somewhere the label denies.
    expect(find.text('Orchestrator'), findsOneWidget);
    expect(find.text('Local'), findsNothing);
    expect(control.state.error, isNotNull);
  });
}

class _FakeChatControl extends ChatControl {
  _FakeChatControl(this.value);
  final ChatControlState value;
  final List<String> requestedModes = [];

  @override
  ChatControlState build(String conversationId) => value;

  @override
  Future<void> load() async {}

  @override
  Future<void> setMode(String mode, {String? agentId}) async {
    requestedModes.add(mode);
  }
}

/// Stands in for Rust refusing the lane: records the error the way the real
/// notifier does and leaves `policy` untouched.
class _RejectingChatControl extends ChatControl {
  _RejectingChatControl(this.value);
  final ChatControlState value;

  @override
  ChatControlState build(String conversationId) => value;

  @override
  Future<void> load() async {}

  @override
  Future<void> setMode(String mode, {String? agentId}) async {
    state = state.copyWith(error: 'no local inference engine is registered');
  }
}
