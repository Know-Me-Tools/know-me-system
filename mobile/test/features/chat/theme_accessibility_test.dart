// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

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
      await tester.pumpWidget(
        MaterialApp(
          home: shad.Theme(
            data: KnowMeTheme.dark,
            child: const Scaffold(
              body: SizedBox(
                width: 400,
                child: ContentBlockView(
                  block: ToolResultBlock(
                    'tool-2',
                    '{"ok":false,"code":"local_tool_requires_confirmation","message":"The local model produced a valid tool-call candidate, but KnowMe did not execute it because the tool is not explicitly read-only, non-destructive, and closed-world.","tool":"files__delete","requiresConfirmation":true,"executionStatus":"not_executed","reasons":["tool_may_be_destructive"],"confirmation":{"status":"required","executionStatus":"not_executed","policy":"manual_confirmation_required_for_mutating_destructive_or_open_world_tools"}}',
                    true,
                  ),
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
      expect(find.byTooltip('Copy tool policy'), findsOneWidget);
    },
  );
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
