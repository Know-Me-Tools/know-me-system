// TJ-ARCH-MOB-001 compliant
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:knowme_poc/core/theme/tokens.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

const _stream = '''
{"version":"v0.9","createSurface":{"surfaceId":"profile","catalogId":"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"}}
{"version":"v0.9","updateComponents":{"surfaceId":"profile","components":[{"id":"root","component":"Column","children":["name","action"]},{"id":"name","component":"Text","text":{"path":"/person/name"}},{"id":"action","component":"Button","child":"actionText","action":{"event":{"name":"reviewProfile","context":{"id":"42"}}}},{"id":"actionText","component":"Text","text":"Review"}]}}
{"version":"v0.9","updateDataModel":{"surfaceId":"profile","path":"/person","value":{"name":"Ada Lovelace"}}}
''';

void main() {
  testWidgets('A2UI v0.9 JSONL renders data bindings and dispatches actions', (
    tester,
  ) async {
    String? action;
    await tester.pumpWidget(
      MaterialApp(
        home: shad.Theme(
          data: KnowMeTheme.light,
          child: Scaffold(
            body: A2uiSurfaceView(
              source: _stream,
              onAction: (event) {
                action = '${event.name}:${event.context['id']}';
              },
            ),
          ),
        ),
      ),
    );

    await tester.pumpAndSettle();
    expect(find.text('Ada Lovelace'), findsOneWidget);
    expect(find.text('Review'), findsOneWidget);
    await tester.tap(find.text('Review'));
    await tester.pump();
    expect(action, 'reviewProfile:42');
  });

  testWidgets('unknown A2UI catalog components fail closed', (tester) async {
    const source = '''
{"version":"v0.9","createSurface":{"surfaceId":"unsafe","catalogId":"custom"}}
{"version":"v0.9","updateComponents":{"surfaceId":"unsafe","components":[{"id":"root","component":"ExecuteScript","source":"alert(1)"}]}}
''';
    await tester.pumpWidget(
      MaterialApp(
        home: shad.Theme(
          data: KnowMeTheme.dark,
          child: const Scaffold(body: A2uiSurfaceView(source: source)),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.textContaining('alert(1)'), findsNothing);
    expect(find.byType(A2uiSurfaceView), findsOneWidget);
  });
}
