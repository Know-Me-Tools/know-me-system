// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'package:knowme_poc/core/theme/tokens.dart';
import 'package:knowme_poc/features/chat/presentation/providers/chat_control_notifier.dart';
import 'package:knowme_poc/features/chat/presentation/widgets/chat_policy_toolbar.dart';
import 'package:knowme_poc/features/settings/domain/settings_snapshot.dart';
import 'package:knowme_poc/features/settings/presentation/providers/settings_notifier.dart';
import 'package:knowme_poc/features/settings/presentation/screens/settings_screen.dart';

const _settingsState = SettingsState(
  snapshot: SettingsSnapshot(
    activeLane: 'uar',
    uarConfig: {'serviceUrl': 'http://127.0.0.1:1906', 'hasApiKey': true},
    providers: [
      {
        'id': 'openai',
        'displayName': 'OpenAI',
        'configured': true,
        'enabled': true,
        'models': [
          {
            'id': 'gpt-5.6-sol',
            'displayName': 'GPT-5.6 Sol',
            'contextWindow': 400000,
            'supportsTools': true,
            'supportsReasoning': true,
            'supportsVision': true,
            'enabled': true,
          },
        ],
      },
      {
        'id': 'anthropic',
        'displayName': 'Anthropic',
        'configured': false,
        'enabled': false,
        'models': <Object>[],
      },
    ],
    configuredProviders: [
      {
        'id': 'openai',
        'displayName': 'OpenAI',
        'enabled': true,
        'hasApiKey': true,
        'defaultModel': 'gpt-5.6-sol',
        'models': [
          {'id': 'gpt-5.6-sol', 'enabled': true},
        ],
      },
    ],
    cloudModel: {'providerId': 'openai', 'modelId': 'gpt-5.6-sol'},
  ),
);

Widget _settings(shad.ThemeData theme) => ProviderScope(
  overrides: [settingsProvider.overrideWith(_FakeSettingsNotifier.new)],
  child: MaterialApp(
    home: shad.Theme(data: theme, child: const SettingsScreen(autoLoad: false)),
  ),
);

void main() {
  for (final theme in <(String, shad.ThemeData)>[
    ('light', KnowMeTheme.light),
    ('dark', KnowMeTheme.dark),
  ]) {
    testWidgets('mobile settings workspace ${theme.$1}', (tester) async {
      tester.view.physicalSize = const Size(390, 844);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(_settings(theme.$2));
      await tester.pumpAndSettle();
      await expectLater(
        find.byType(SettingsScreen),
        matchesGoldenFile('goldens/settings_workspace_${theme.$1}.png'),
      );
    });
  }

  testWidgets('settings categories are reachable on a narrow phone', (
    tester,
  ) async {
    await tester.pumpWidget(_settings(KnowMeTheme.dark));
    await tester.pumpAndSettle();
    expect(find.text('Model providers'), findsWidgets);
    await tester.tap(find.text('Universal Agent Runtime').first);
    await tester.pumpAndSettle();
    expect(find.text('Chat runtime'), findsOneWidget);
    expect(find.text('UAR'), findsWidgets);
    expect(find.text('Local'), findsWidgets);
  });

  testWidgets('conversation toolbar exposes UAR, agent and context controls', (
    tester,
  ) async {
    const state = ChatControlState(
      catalog: {
        // Agents are shaped like real AgentArtifacts — the display name lives
        // at `metadata.title`, not a flat `name`. A previous fixture used the
        // flat key, which hid a bug where every embedded surface fell back to
        // showing raw agent ids.
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
        },
        'skills': [
          {'skill_id': 'drafting', 'title': 'Drafting'},
        ],
        'tools': {
          'mcp_servers': ['calendar'],
        },
        'knowledge_bases': [
          {'id': 'personal', 'name': 'Personal'},
        ],
      },
      policy: {
        'chat_mode': 'uar',
        'skills': {'mode': 'inherit', 'ids': <String>[]},
        'mcp_servers': {'mode': 'all', 'ids': <String>[]},
        'knowledge_bases': {
          'mode': 'selected',
          'ids': ['personal'],
        },
        'memory_enabled': true,
      },
    );
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          chatControlProvider('conversation')
              .overrideWith(() => _FakeChatControl(state)),
        ],
        child: MaterialApp(
          home: shad.Theme(
            data: KnowMeTheme.dark,
            child: const Scaffold(
              body: ChatPolicyToolbar(
                conversationId: 'conversation',
                autoLoad: false,
              ),
            ),
          ),
        ),
      ),
    );
    // One mode trigger labelled with the current selection — not a fixed
    // Orchestrator/Agent/Local button row. The agent list grows, so the modes
    // live in a sheet the trigger opens.
    expect(find.text('Orchestrator'), findsOneWidget);
    expect(find.text('Agent'), findsNothing);
    expect(find.text('Local'), findsNothing);
    expect(find.text('Skills inherit'), findsOneWidget);
    await tester.drag(find.byType(ListView).last, const Offset(-650, 0));
    await tester.pumpAndSettle();
    expect(find.text('MCP all'), findsOneWidget);
    expect(find.text('Knowledge 1'), findsOneWidget);
    expect(find.text('Memory on'), findsOneWidget);
  });
}

class _FakeSettingsNotifier extends SettingsNotifier {
  @override
  SettingsState build() => _settingsState;

  @override
  Future<void> load() async {}
}

class _FakeChatControl extends ChatControl {
  _FakeChatControl(this.value);
  final ChatControlState value;

  @override
  ChatControlState build(String conversationId) => value;

  @override
  Future<void> load() async {}
}
