// TJ-ARCH-MOB-001 compliant
import 'package:alchemist/alchemist.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';

void main() {
  goldenTest(
    'ContentBlockView renders core variants',
    fileName: 'content_block_view',
    builder: () => GoldenTestGroup(
      children: [
        GoldenTestScenario(
          name: 'text',
          child: const ContentBlockView(block: TextBlock('Hello world')),
        ),
        GoldenTestScenario(
          name: 'thinking',
          child: const ContentBlockView(
            block: ThinkingBlock('reasoning…'),
          ),
        ),
        GoldenTestScenario(
          name: 'code',
          child: const ContentBlockView(
            block: CodeBlock('dart', 'void main() {}'),
          ),
        ),
        GoldenTestScenario(
          name: 'divider',
          child: const ContentBlockView(block: DividerBlock()),
        ),
      ],
    ),
  );
}
