// TJ-ARCH-MOB-001 compliant
// Safe native renderer for Google A2UI v0.9 JSONL. The server supplies data,
// never executable UI code. Only this explicit catalog can materialize widgets.
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:markdown_widget/markdown_widget.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

typedef A2uiActionCallback =
    void Function(
      String surfaceId,
      String actionName,
      Map<String, dynamic> context,
    );

/// Renders an A2UI v0.9 message stream embedded in an artifact block.
///
/// Supported envelopes are createSurface, updateComponents, updateDataModel,
/// and deleteSurface. Components are resolved from the protocol's flat
/// adjacency list and unknown catalog entries fail closed as visible notices.
class A2uiSurfaceView extends StatefulWidget {
  const A2uiSurfaceView({required this.source, this.onAction, super.key});

  final String source;
  final A2uiActionCallback? onAction;

  @override
  State<A2uiSurfaceView> createState() => _A2uiSurfaceViewState();
}

class _A2uiSurfaceViewState extends State<A2uiSurfaceView> {
  late Map<String, _Surface> _surfaces;
  String? _error;

  @override
  void initState() {
    super.initState();
    _decode();
  }

  @override
  void didUpdateWidget(covariant A2uiSurfaceView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.source != widget.source) _decode();
  }

  void _decode() {
    _surfaces = {};
    _error = null;
    try {
      for (final message in _messages(widget.source)) {
        _apply(message);
      }
    } on Object catch (error) {
      _error = 'This A2UI surface could not be rendered: $error';
    }
  }

  Iterable<Map<String, dynamic>> _messages(String source) sync* {
    final trimmed = source.trim();
    if (trimmed.isEmpty) return;
    if (trimmed.startsWith('[')) {
      for (final item in jsonDecode(trimmed) as List<dynamic>) {
        yield Map<String, dynamic>.from(item as Map);
      }
      return;
    }
    for (final line in const LineSplitter().convert(trimmed)) {
      if (line.trim().isNotEmpty) {
        yield Map<String, dynamic>.from(jsonDecode(line) as Map);
      }
    }
  }

  void _apply(Map<String, dynamic> message) {
    if (message['createSurface'] case final Map<dynamic, dynamic> raw) {
      final body = Map<String, dynamic>.from(raw);
      final id = body['surfaceId'] as String;
      _surfaces[id] = _Surface(
        id: id,
        catalogId: body['catalogId'] as String? ?? 'basic',
      );
      return;
    }
    if (message['updateComponents'] case final Map<dynamic, dynamic> raw) {
      final body = Map<String, dynamic>.from(raw);
      final surface = _requireSurface(body['surfaceId'] as String);
      for (final item in body['components'] as List<dynamic>? ?? const []) {
        final component = Map<String, dynamic>.from(item as Map);
        surface.components[component['id'] as String] = component;
      }
      return;
    }
    if (message['updateDataModel'] case final Map<dynamic, dynamic> raw) {
      final body = Map<String, dynamic>.from(raw);
      final surface = _requireSurface(body['surfaceId'] as String);
      _writePointer(
        surface.data,
        body['path'] as String? ?? '/',
        body['value'],
      );
      return;
    }
    if (message['deleteSurface'] case final Map<dynamic, dynamic> raw) {
      _surfaces.remove(raw['surfaceId'] as String);
      return;
    }
    throw const FormatException('unknown A2UI message envelope');
  }

  _Surface _requireSurface(String id) => _surfaces[id] ??= _Surface(
    id: id,
    catalogId: 'https://a2ui.org/catalogs/basic',
  );

  @override
  Widget build(BuildContext context) {
    if (_error case final error?) {
      return Semantics(
        liveRegion: true,
        child: Text(
          error,
          style: TextStyle(
            color: shad.Theme.of(context).colorScheme.destructive,
          ),
        ),
      );
    }
    if (_surfaces.isEmpty) {
      return const Text('This A2UI surface is empty.');
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (final surface in _surfaces.values)
          Semantics(
            container: true,
            label: 'Interactive agent interface',
            child: _component(surface, 'root', <String>{}, 0),
          ),
      ],
    );
  }

  Widget _component(
    _Surface surface,
    String id,
    Set<String> ancestors,
    int depth,
  ) {
    if (depth > 32 || ancestors.contains(id)) {
      return const _Unsupported(label: 'Invalid recursive A2UI component');
    }
    final node = surface.components[id];
    if (node == null) {
      return _Unsupported(label: 'Waiting for component “$id”…');
    }
    final next = {...ancestors, id};
    Widget child(String childId) =>
        _component(surface, childId, next, depth + 1);
    List<Widget> children() => [
      for (final childId in node['children'] as List<dynamic>? ?? const [])
        child(childId as String),
    ];
    final type = node['component'] as String? ?? '';
    return KeyedSubtree(
      key: ValueKey('${surface.id}:$id'),
      child: switch (type) {
        'Text' => MarkdownBlock(
          data: _stringValue(surface, node['text']),
          config: _a2uiMarkdownConfig(context),
        ),
        'Column' => Column(
          crossAxisAlignment: _crossAxis(node['align'] as String?),
          mainAxisSize: MainAxisSize.min,
          children: _spaced(children()),
        ),
        'Row' => Wrap(
          spacing: 12,
          runSpacing: 8,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: children(),
        ),
        'List' => Column(
          mainAxisSize: MainAxisSize.min,
          children: _spaced(children()),
        ),
        'Card' => Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: shad.Theme.of(context).colorScheme.card,
            borderRadius: BorderRadius.circular(14),
          ),
          child: child(node['child'] as String),
        ),
        'Icon' => Icon(_icon(node['name'] as String?)),
        'Image' => _A2uiMediaPlaceholder(
          icon: Icons.image_outlined,
          label: node['alt'] as String? ?? 'A2UI image',
          source: _stringValue(surface, node['url']),
        ),
        'Video' || 'AudioPlayer' => _A2uiMediaPlaceholder(
          icon: type == 'Video'
              ? Icons.movie_outlined
              : Icons.audio_file_outlined,
          label: type,
          source: _stringValue(surface, node['url']),
        ),
        'Button' => shad.PrimaryButton(
          onPressed: widget.onAction == null
              ? null
              : () => _dispatch(surface, node['action']),
          child: switch (node['child']) {
            final String childId => child(childId),
            _ => Text(node['text'] as String? ?? 'Continue'),
          },
        ),
        'TextField' => shad.TextField(
          placeholder: Text(node['label'] as String? ?? ''),
          onChanged: (value) => _updateBinding(surface, node['value'], value),
        ),
        'CheckBox' => _A2uiCheckBox(
          label: node['label'] as String? ?? '',
          value: _boolValue(surface, node['value']),
          onChanged: (value) =>
              setState(() => _updateBinding(surface, node['value'], value)),
        ),
        'ChoicePicker' => _A2uiChoicePicker(
          label: node['label'] as String? ?? 'Choose an option',
          options: [
            for (final option in node['options'] as List<dynamic>? ?? const [])
              Map<String, dynamic>.from(option as Map),
          ],
        ),
        'Divider' => const SizedBox(height: 16),
        'Tabs' || 'Modal' => Column(
          mainAxisSize: MainAxisSize.min,
          children: _spaced(children()),
        ),
        _ => _Unsupported(label: 'Unsupported A2UI component: $type'),
      },
    );
  }

  void _dispatch(_Surface surface, dynamic rawAction) {
    if (rawAction is! Map) return;
    final action = Map<String, dynamic>.from(rawAction);
    final event = action['event'] is Map
        ? Map<String, dynamic>.from(action['event'] as Map)
        : action;
    widget.onAction?.call(
      surface.id,
      event['name'] as String? ?? 'action',
      Map<String, dynamic>.from(event['context'] as Map? ?? const {}),
    );
  }

  void _updateBinding(_Surface surface, dynamic binding, dynamic value) {
    if (binding is Map && binding['path'] is String) {
      _writePointer(surface.data, binding['path'] as String, value);
    }
  }

  String _stringValue(_Surface surface, dynamic value) {
    final resolved = _resolve(surface, value);
    return resolved?.toString() ?? '';
  }

  bool _boolValue(_Surface surface, dynamic value) =>
      _resolve(surface, value) == true;

  dynamic _resolve(_Surface surface, dynamic value) {
    if (value is Map && value['path'] is String) {
      return _readPointer(surface.data, value['path'] as String);
    }
    return value;
  }
}

class _Surface {
  _Surface({required this.id, required this.catalogId});
  final String id;
  final String catalogId;
  final Map<String, Map<String, dynamic>> components = {};
  final Map<String, dynamic> data = {};
}

class _Unsupported extends StatelessWidget {
  const _Unsupported({required this.label});
  final String label;
  @override
  Widget build(BuildContext context) => Text(
    label,
    style: TextStyle(color: shad.Theme.of(context).colorScheme.mutedForeground),
  );
}

class _A2uiCheckBox extends StatelessWidget {
  const _A2uiCheckBox({
    required this.label,
    required this.value,
    required this.onChanged,
  });
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => Semantics(
    checked: value,
    child: InkWell(
      onTap: () => onChanged(!value),
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Row(
          children: [
            Icon(value ? Icons.check_box : Icons.check_box_outline_blank),
            const SizedBox(width: 8),
            Expanded(child: Text(label)),
          ],
        ),
      ),
    ),
  );
}

class _A2uiChoicePicker extends StatelessWidget {
  const _A2uiChoicePicker({required this.label, required this.options});
  final String label;
  final List<Map<String, dynamic>> options;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(label),
      const SizedBox(height: 8),
      Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          for (final option in options)
            shad.OutlineButton(
              onPressed: null,
              child: Text(
                option['label'] as String? ?? option['value'].toString(),
              ),
            ),
        ],
      ),
    ],
  );
}

class _A2uiMediaPlaceholder extends StatelessWidget {
  const _A2uiMediaPlaceholder({
    required this.icon,
    required this.label,
    required this.source,
  });
  final IconData icon;
  final String label;
  final String source;
  @override
  Widget build(BuildContext context) => Semantics(
    image: true,
    label: label,
    child: Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: shad.Theme.of(context).colorScheme.secondary,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Icon(icon),
          const SizedBox(width: 10),
          Expanded(
            child: Text(source, maxLines: 2, overflow: TextOverflow.ellipsis),
          ),
        ],
      ),
    ),
  );
}

List<Widget> _spaced(List<Widget> widgets) => [
  for (var index = 0; index < widgets.length; index++) ...[
    if (index > 0) const SizedBox(height: 10),
    widgets[index],
  ],
];

CrossAxisAlignment _crossAxis(String? value) => switch (value) {
  'center' => CrossAxisAlignment.center,
  'end' => CrossAxisAlignment.end,
  'stretch' => CrossAxisAlignment.stretch,
  _ => CrossAxisAlignment.start,
};

IconData _icon(String? name) => switch (name) {
  'mail' => Icons.mail_outline,
  'calendar' => Icons.calendar_today_outlined,
  'person' => Icons.person_outline,
  'location' => Icons.location_on_outlined,
  'check' => Icons.check,
  'warning' => Icons.warning_amber_outlined,
  _ => Icons.widgets_outlined,
};

MarkdownConfig _a2uiMarkdownConfig(BuildContext context) {
  final colors = shad.Theme.of(context).colorScheme;
  final dark = colors.background.computeLuminance() < 0.5;
  return (dark ? MarkdownConfig.darkConfig : MarkdownConfig.defaultConfig).copy(
    configs: [
      PConfig(
        textStyle: TextStyle(
          color: colors.foreground,
          fontSize: 16,
          height: 1.5,
        ),
      ),
      for (final heading in const [
        (MarkdownTag.h1, 30.0),
        (MarkdownTag.h2, 25.0),
        (MarkdownTag.h3, 22.0),
        (MarkdownTag.h4, 19.0),
        (MarkdownTag.h5, 17.0),
        (MarkdownTag.h6, 16.0),
      ])
        _A2uiHeadingConfig(
          tag: heading.$1.name,
          style: TextStyle(
            color: colors.foreground,
            fontSize: heading.$2,
            height: 1.25,
            fontWeight: FontWeight.w700,
          ),
        ),
    ],
  );
}

class _A2uiHeadingConfig extends HeadingConfig {
  const _A2uiHeadingConfig({required this.tag, required this.style});
  @override
  final String tag;
  @override
  final TextStyle style;
}

dynamic _readPointer(Map<String, dynamic> root, String pointer) {
  dynamic current = root;
  for (final segment in _segments(pointer)) {
    if (current is! Map<String, dynamic>) return null;
    current = current[segment];
  }
  return current;
}

void _writePointer(Map<String, dynamic> root, String pointer, dynamic value) {
  final segments = _segments(pointer);
  if (segments.isEmpty) {
    root
      ..clear()
      ..addAll(
        value is Map ? Map<String, dynamic>.from(value) : {'value': value},
      );
    return;
  }
  var current = root;
  for (final segment in segments.take(segments.length - 1)) {
    current =
        current.putIfAbsent(segment, () => <String, dynamic>{})
            as Map<String, dynamic>;
  }
  final leaf = segments.last;
  if (value == null) {
    current.remove(leaf);
  } else {
    current[leaf] = value;
  }
}

List<String> _segments(String pointer) => pointer == '/' || pointer.isEmpty
    ? const []
    : pointer
          .split('/')
          .skip(1)
          .map((part) => part.replaceAll('~1', '/').replaceAll('~0', '~'))
          .toList();
