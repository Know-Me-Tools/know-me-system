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

  testWidgets('rich block actions are labeled and at least 44 pixels',
      (tester) async {
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
    expect(size.width, greaterThanOrEqualTo(44));
    expect(size.height, greaterThanOrEqualTo(44));
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
