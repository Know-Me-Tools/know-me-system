// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'package:knowme_poc/bridge/a2ui/a2ui_event.dart';
import 'package:knowme_poc/core/theme/tokens.dart';

void main() {
  test('light and dark semantic text roles meet WCAG AA', () {
    for (final palette in [KnowMePalette.light, KnowMePalette.dark]) {
      expect(
        _contrast(palette.text, palette.canvas),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(palette.subtleText, palette.canvas),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(palette.faintText, palette.canvas),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(palette.onAccent, palette.accent),
        greaterThanOrEqualTo(4.5),
      );
      expect(
        _contrast(palette.text, palette.raised),
        greaterThanOrEqualTo(4.5),
      );
    }
  });

  test('content blocks decode Rust snake-case tool wire fields', () {
    final toolUse = ContentBlock.fromJson({
      'type': 'toolUse',
      'id': 'litert-tool-0',
      'name': 'knowme_diagnostics__approval_required_echo',
      'input_json': '{"phrase":"approval-ok"}',
    });
    expect(toolUse, isA<ToolUseBlock>());
    expect((toolUse as ToolUseBlock).inputJson, '{"phrase":"approval-ok"}');

    final toolResult = ContentBlock.fromJson({
      'type': 'toolResult',
      'tool_use_id': 'litert-tool-0',
      'output_json': '{"ok":false}',
      'is_error': true,
    });
    expect(toolResult, isA<ToolResultBlock>());
    expect((toolResult as ToolResultBlock).toolUseId, 'litert-tool-0');
    expect(toolResult.isError, isTrue);
  });

  test('opaque Rust errors are sanitized for user-visible bubbles', () {
    final error = ChatRunError.fromJson({
      'code': 'tool_approval_start_failed',
      'phase': 'tool_execution',
      'message': "Instance of 'CoreErrorImpl'",
      'retryable': false,
    });

    expect(error.message, isNot(contains('CoreErrorImpl')));
    expect(error.displayMessage, contains('approval may have expired'));
  });

  testWidgets('rich block actions are labeled and at least 48 pixels', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: shad.Theme(
          data: KnowMeTheme.dark,
          child: const Scaffold(
            body: Align(
              alignment: Alignment.topLeft,
              child: SizedBox(
                width: 400,
                child: ColoredBox(
                  color: Color(0xFF0B0F14),
                  child: ContentBlockView(
                    block: CodeBlock('dart', 'void main() {}'),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );

    final copy = find.byTooltip('Copy code');
    expect(copy, findsOneWidget);
    final size = tester.getSize(copy);
    expect(size.width, greaterThanOrEqualTo(48));
    expect(size.height, greaterThanOrEqualTo(48));
    await expectLater(tester, meetsGuideline(textContrastGuideline));
    await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
  });

  testWidgets(
    'confirmation-required tool results render as explicit policy cards',
    (tester) async {
      Map<String, Object?>? interaction;
      await tester.pumpWidget(
        MaterialApp(
          home: shad.Theme(
            data: KnowMeTheme.dark,
            child: Scaffold(
              body: SizedBox(
                width: 400,
                child: ContentBlockView(
                  block: const ToolResultBlock(
                    'tool-2',
                    '{"ok":false,"code":"local_tool_requires_confirmation","message":"The local model produced a valid tool-call candidate, but KnowMe did not execute it because the tool is not explicitly read-only, non-destructive, and closed-world.","tool":"files__delete","approvalId":"approval-1","requiresConfirmation":true,"executionStatus":"not_executed","reasons":["tool_may_be_destructive"],"confirmation":{"status":"required","approvalId":"approval-1","executionStatus":"not_executed","policy":"manual_confirmation_required_for_mutating_destructive_or_open_world_tools"}}',
                    true,
                  ),
                  onInteraction: (value) => interaction = value,
                ),
              ),
            ),
          ),
        ),
      );

      expect(find.text('Approval required'), findsOneWidget);
      expect(find.text('files__delete'), findsOneWidget);
      expect(find.text('May be destructive'), findsOneWidget);
      expect(find.textContaining('Status: not executed'), findsOneWidget);
      expect(
        find.textContaining('Approvals are process-local'),
        findsOneWidget,
      );
      expect(find.byTooltip('Copy tool policy'), findsOneWidget);
      expect(find.text('Approve and run'), findsOneWidget);
      await tester.tap(find.text('Approve and run'));
      expect(interaction?['kind'], 'local_tool_approve');
      expect(interaction?['approvalId'], 'approval-1');
      expect(interaction?['toolUseId'], 'tool-2');
    },
  );

  testWidgets('long tool names stay on one header line', (tester) async {
    const longToolName = 'knowme_diagnostics__approval_required_echo';
    await tester.pumpWidget(
      MaterialApp(
        home: shad.Theme(
          data: KnowMeTheme.dark,
          child: const Scaffold(
            body: SizedBox(
              width: 320,
              child: ContentBlockView(
                block: ToolUseBlock(longToolName, longToolName, '{"ok":true}'),
              ),
            ),
          ),
        ),
      ),
    );

    final title = tester.widget<Text>(find.text(longToolName));
    expect(title.maxLines, 1);
    expect(title.overflow, TextOverflow.ellipsis);
  });
}

double _contrast(Color first, Color second) {
  final lighter = first.computeLuminance() > second.computeLuminance()
      ? first.computeLuminance()
      : second.computeLuminance();
  final darker = first.computeLuminance() > second.computeLuminance()
      ? second.computeLuminance()
      : first.computeLuminance();
  return (lighter + 0.05) / (darker + 0.05);
}
