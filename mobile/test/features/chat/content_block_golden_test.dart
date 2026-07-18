// TJ-ARCH-MOB-001 compliant
import 'package:alchemist/alchemist.dart';
import 'package:flutter/material.dart';
import 'package:gen_ui_widgets/gen_ui_widgets.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'package:knowme_poc/core/theme/tokens.dart';

const _a2ui = '''
{"version":"v0.9","createSurface":{"surfaceId":"welcome","catalogId":"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"}}
{"version":"v0.9","updateComponents":{"surfaceId":"welcome","components":[{"id":"root","component":"Card","child":"body"},{"id":"body","component":"Column","children":["title","summary","action"],"align":"stretch"},{"id":"title","component":"Text","text":"## Your day"},{"id":"summary","component":"Text","text":"Two memories are ready to review."},{"id":"action","component":"Button","text":"Review memories"}]}}
''';

const _blocks = <ContentBlock>[
  TextBlock('**Markdown** with a [source](https://example.com).'),
  ThinkingBlock('I compared the available memories.'),
  CodeBlock('dart', 'void main() => print("Hello");'),
  CitationBlock('KnowMe memory #42', 'The source excerpt remains inspectable.'),
  MemoryBlock('read', 'favorite_color', 'ember'),
  ToolUseBlock('tool-1', 'memory_search', '{"query":"favorite color"}'),
  ToolResultBlock('tool-1', '{"matches":2}', false),
  SkillBlock('memory-recall', 'completed'),
  ArtifactBlock('artifact-1', 'markdown', '# Generated note\nReady to save.'),
  ImageBlock(null, null, 'image/png'),
  DividerBlock(),
  ArtifactBlock('surface-1', 'a2ui', _a2ui),
  ArtifactBlock(
    'state-1',
    'agui_state_delta',
    '[{"op":"replace","path":"/status","value":"ready"}]',
  ),
  ArtifactBlock(
    'confirm-1',
    'agui_confirmation',
    '{"title":"Save this memory?","message":"KnowMe will remember this across conversations.","choices":["Not now","Save memory"]}',
  ),
];

void main() {
  for (final theme in <(String, shad.ThemeData)>[
    ('light', KnowMeTheme.light),
    ('dark', KnowMeTheme.dark),
  ]) {
    goldenTest(
      'ContentBlockView renders exhaustive ${theme.$1} matrix',
      fileName: 'content_block_view_${theme.$1}',
      constraints: const BoxConstraints.tightFor(width: 420),
      builder: () => GoldenTestGroup(
        columns: 1,
        children: [
          for (var index = 0; index < _blocks.length; index++)
            GoldenTestScenario(
              name: '${index + 1} ${_blocks[index].runtimeType}',
              child: shad.Theme(
                data: theme.$2,
                child: ColoredBox(
                  color: theme.$2.colorScheme.background,
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: DefaultTextStyle(
                      style: TextStyle(color: theme.$2.colorScheme.foreground),
                      child: ContentBlockView(block: _blocks[index]),
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
