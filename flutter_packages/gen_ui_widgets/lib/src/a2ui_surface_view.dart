// TJ-ARCH-MOB-001 compliant
// Safe native renderer for Google A2UI v0.9 JSONL. Protocol parsing, surface
// lifecycle, schema validation, reactive data binding, client functions, and
// action construction are delegated to Flutter's official GenUI SDK.
import 'dart:async';
import 'dart:convert';

import 'package:audioplayers/audioplayers.dart' as audio;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:genui/genui.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart' as shad;

import 'content_block_view.dart';

/// A typed, transport-ready A2UI v0.9 user action.
@immutable
class A2uiAction {
  const A2uiAction({
    required this.surfaceId,
    required this.name,
    required this.sourceComponentId,
    required this.timestamp,
    required this.context,
    this.clientDataModel,
  });

  final String surfaceId;
  final String name;
  final String sourceComponentId;
  final DateTime timestamp;
  final Map<String, Object?> context;

  /// Present only when the surface opted into `sendDataModel`.
  final Map<String, Object?>? clientDataModel;

  Map<String, Object?> toJson() => {
    'version': 'v0.9',
    'action': {
      'surfaceId': surfaceId,
      'name': name,
      'sourceComponentId': sourceComponentId,
      'timestamp': timestamp.toUtc().toIso8601String(),
      'context': context,
    },
    if (clientDataModel != null) 'a2uiClientDataModel': clientDataModel,
  };
}

typedef A2uiActionCallback = FutureOr<void> Function(A2uiAction action);

/// Renders an A2UI v0.9 message stream embedded in an artifact block.
///
/// The official GenUI engine owns the protocol state machine. This widget owns
/// only its rendering lifecycle and forwards typed user intent to the host. The
/// host must send actions through Riverpod -> Repository -> Rust FFI.
class A2uiSurfaceView extends StatefulWidget {
  const A2uiSurfaceView({
    required this.source,
    this.onAction,
    this.remoteResolver,
    super.key,
  });

  final String source;
  final A2uiActionCallback? onAction;
  final RemoteContentResolver? remoteResolver;

  @override
  State<A2uiSurfaceView> createState() => _A2uiSurfaceViewState();
}

class _A2uiSurfaceViewState extends State<A2uiSurfaceView> {
  late SurfaceController _controller;
  late A2uiTransportAdapter _transport;
  StreamSubscription<Object?>? _messageSubscription;
  StreamSubscription<Object?>? _surfaceSubscription;
  StreamSubscription<ChatMessage>? _submitSubscription;
  List<String> _surfaceIds = const [];
  final Map<String, bool> _sendDataModelBySurface = {};
  String? _error;

  @override
  void initState() {
    super.initState();
    _start();
  }

  @override
  void didUpdateWidget(covariant A2uiSurfaceView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.source != widget.source) {
      _stop();
      _start();
    }
  }

  void _start() {
    _error = null;
    _surfaceIds = const [];
    _indexSurfacePrivacyFlags(widget.source);
    final basicCatalog = BasicCatalogItems.asNoAssetCatalog().copyWith(
      newItems: [_safeImageItem(), _safeVideoItem(), _safeAudioItem()],
    );
    _controller = SurfaceController(
      // Image/audio/video are supplied by the Rust media resolver in a custom
      // catalog later in the pipeline. Excluding the stock network-backed
      // widgets here preserves the Rust-owned networking invariant.
      catalogs: [
        basicCatalog,
        // Kept for compatibility with early v0.9 examples and persisted agent
        // output. Both IDs use the same official, schema-validated catalog.
        basicCatalog.copyWith(
          catalogId:
              'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json',
        ),
      ],
    );
    _transport = A2uiTransportAdapter();
    _messageSubscription = _transport.incomingMessages.listen(
      _controller.handleMessage,
      onError: _showError,
    );
    _surfaceSubscription = _controller.surfaceUpdates.listen(
      (_) => _refreshSurfaces(),
      onError: _showError,
    );
    _submitSubscription = _controller.onSubmit.listen(
      _forwardInteraction,
      onError: _showError,
    );

    try {
      // A final newline gives the JSONL parser an unambiguous frame boundary.
      _transport.addChunk('${widget.source.trim()}\n');
    } on Object catch (error, stackTrace) {
      _showError(error, stackTrace);
    }
  }

  CatalogItem _safeImageItem() => CatalogItem(
    name: 'Image',
    dataSchema: BasicCatalogItems.image.dataSchema,
    widgetBuilder: (itemContext) {
      final data = Map<String, Object?>.from(itemContext.data as Map);
      return BoundString(
        dataContext: itemContext.dataContext,
        value: data['url'],
        builder: (_, value) => ContentBlockImage(
          url: value,
          dataBase64: null,
          mime: 'image/unknown',
          alt: 'Interactive surface image',
          remoteResolver: widget.remoteResolver,
        ),
      );
    },
  );

  CatalogItem _safeVideoItem() => CatalogItem(
    name: 'Video',
    dataSchema: BasicCatalogItems.video.dataSchema,
    widgetBuilder: (itemContext) {
      final data = Map<String, Object?>.from(itemContext.data as Map);
      return BoundString(
        dataContext: itemContext.dataContext,
        value: data['url'],
        builder: (_, value) => value == null || value.isEmpty
            ? const SizedBox.shrink()
            : ContentBlockVideo(
                source: value,
                remoteResolver: widget.remoteResolver,
              ),
      );
    },
  );

  CatalogItem _safeAudioItem() => CatalogItem(
    name: 'AudioPlayer',
    dataSchema: BasicCatalogItems.audioPlayer.dataSchema,
    widgetBuilder: (itemContext) {
      final data = Map<String, Object?>.from(itemContext.data as Map);
      return BoundString(
        dataContext: itemContext.dataContext,
        value: data['url'],
        builder: (_, url) => BoundString(
          dataContext: itemContext.dataContext,
          value: data['description'],
          builder: (_, description) => _SafeAudioPlayer(
            url: url,
            description: description,
            remoteResolver: widget.remoteResolver,
          ),
        ),
      );
    },
  );

  void _refreshSurfaces() {
    if (!mounted) return;
    setState(() {
      _surfaceIds = List.unmodifiable(_controller.activeSurfaceIds);
    });
  }

  void _forwardInteraction(ChatMessage message) {
    for (final part in message.parts.uiInteractionParts) {
      try {
        final envelope = Map<String, Object?>.from(
          jsonDecode(part.interaction) as Map,
        );
        final rawAction = envelope['action'];
        if (rawAction is! Map) continue;
        final action = Map<String, Object?>.from(rawAction);
        final surfaceId = action['surfaceId']?.toString() ?? '';
        if (surfaceId.isEmpty) continue;
        final Object? dataModel = _sendDataModelBySurface[surfaceId] == true
            ? _controller
                  .contextFor(surfaceId)
                  .dataModel
                  .getValue<Object?>(DataPath.root)
            : null;
        final timestamp =
            DateTime.tryParse(action['timestamp']?.toString() ?? '') ??
            DateTime.now().toUtc();
        final context = action['context'] is Map
            ? Map<String, Object?>.from(action['context'] as Map)
            : <String, Object?>{};
        final callback = widget.onAction;
        if (callback != null) {
          unawaited(
            Future.sync(
              () => callback(
                A2uiAction(
                  surfaceId: surfaceId,
                  name: action['name']?.toString() ?? 'action',
                  sourceComponentId:
                      action['sourceComponentId']?.toString() ?? 'unknown',
                  timestamp: timestamp,
                  context: context,
                  clientDataModel: dataModel == null
                      ? null
                      : {
                          'version': 'v0.9',
                          'surfaces': {surfaceId: dataModel},
                        },
                ),
              ),
            ).catchError((Object error, StackTrace stackTrace) {
              _showError(error, stackTrace);
            }),
          );
        }
      } on Object catch (error, stackTrace) {
        _showError(error, stackTrace);
      }
    }
  }

  void _indexSurfacePrivacyFlags(String source) {
    _sendDataModelBySurface.clear();
    for (final line in const LineSplitter().convert(source)) {
      try {
        final envelope = jsonDecode(line);
        if (envelope is! Map) continue;
        final create = envelope['createSurface'];
        if (create is Map) {
          final surfaceId = create['surfaceId'];
          if (surfaceId is String && surfaceId.isNotEmpty) {
            _sendDataModelBySurface[surfaceId] =
                create['sendDataModel'] == true;
          }
        }
        final delete = envelope['deleteSurface'];
        if (delete is Map && delete['surfaceId'] is String) {
          _sendDataModelBySurface.remove(delete['surfaceId']);
        }
      } on FormatException {
        // The official transport reports malformed frames. This pass only
        // indexes the opt-in privacy flag and never accepts invalid UI.
      }
    }
  }

  void _showError(Object error, [StackTrace? stackTrace]) {
    if (!mounted) return;
    setState(() {
      _error = 'This A2UI surface could not be rendered safely.';
    });
  }

  void _stop() {
    unawaited(_messageSubscription?.cancel());
    unawaited(_surfaceSubscription?.cancel());
    unawaited(_submitSubscription?.cancel());
    _transport.dispose();
    _controller.dispose();
  }

  @override
  void dispose() {
    _stop();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = shad.Theme.of(context).colorScheme;
    final brightness = colors.background.computeLuminance() < 0.5
        ? Brightness.dark
        : Brightness.light;
    final materialTheme = ThemeData(
      brightness: brightness,
      colorScheme:
          ColorScheme.fromSeed(
            seedColor: colors.primary,
            brightness: brightness,
          ).copyWith(
            primary: colors.primary,
            onPrimary: colors.primaryForeground,
            surface: colors.card,
            onSurface: colors.foreground,
            error: colors.destructive,
          ),
      scaffoldBackgroundColor: colors.background,
      dividerColor: Colors.transparent,
      cardTheme: CardThemeData(
        color: colors.card,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide.none,
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          elevation: 0,
          shadowColor: Colors.transparent,
          minimumSize: const Size(48, 48),
        ),
      ),
      inputDecorationTheme: const InputDecorationTheme(
        border: InputBorder.none,
        enabledBorder: InputBorder.none,
        focusedBorder: InputBorder.none,
      ),
    );
    if (_error case final error?) {
      return Semantics(
        liveRegion: true,
        child: Text(error, style: TextStyle(color: colors.destructive)),
      );
    }
    if (_surfaceIds.isEmpty) {
      return Semantics(
        liveRegion: true,
        label: 'Interactive agent interface is loading',
        child: const Padding(
          padding: EdgeInsets.all(16),
          child: Center(child: CircularProgressIndicator(strokeWidth: 2)),
        ),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (final surfaceId in _surfaceIds)
          Semantics(
            container: true,
            label: 'Interactive agent interface',
            child: Theme(
              data: materialTheme,
              child: Surface(
                surfaceContext: _controller.contextFor(surfaceId),
                defaultBuilder: (_) => const SizedBox.shrink(),
              ),
            ),
          ),
      ],
    );
  }
}

class _SafeAudioPlayer extends StatefulWidget {
  const _SafeAudioPlayer({
    required this.url,
    required this.description,
    required this.remoteResolver,
  });

  final String? url;
  final String? description;
  final RemoteContentResolver? remoteResolver;

  @override
  State<_SafeAudioPlayer> createState() => _SafeAudioPlayerState();
}

class _SafeAudioPlayerState extends State<_SafeAudioPlayer> {
  final audio.AudioPlayer _player = audio.AudioPlayer();
  StreamSubscription<audio.PlayerState>? _stateSubscription;
  bool _ready = false;
  bool _playing = false;
  Object? _error;

  @override
  void initState() {
    super.initState();
    _stateSubscription = _player.onPlayerStateChanged.listen((state) {
      if (mounted) {
        setState(() => _playing = state == audio.PlayerState.playing);
      }
    });
    unawaited(_initialize());
  }

  Future<void> _initialize() async {
    try {
      final url = widget.url;
      final resolver = widget.remoteResolver;
      if (url == null || url.isEmpty || resolver == null) {
        throw const FormatException('Audio source is unavailable');
      }
      final resolved = await resolver(url, RemoteContentKind.audio);
      final path = resolved.localPath;
      if (path == null) {
        throw const FormatException('Audio cache path is unavailable');
      }
      await _player.setSource(audio.DeviceFileSource(path));
      if (mounted) setState(() => _ready = true);
    } on Object catch (error) {
      if (mounted) setState(() => _error = error);
    }
  }

  @override
  void dispose() {
    unawaited(_stateSubscription?.cancel());
    unawaited(_player.dispose());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    if (_error != null) {
      return Semantics(
        label: 'Audio unavailable',
        child: const Icon(Icons.audio_file_outlined),
      );
    }
    return Semantics(
      container: true,
      label: widget.description ?? 'Audio player',
      child: Row(
        children: [
          IconButton(
            tooltip: _playing ? 'Pause audio' : 'Play audio',
            constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
            onPressed: !_ready
                ? null
                : () => _playing ? _player.pause() : _player.resume(),
            icon: _ready
                ? Icon(_playing ? Icons.pause : Icons.play_arrow)
                : const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
          ),
          Expanded(
            child: Text(
              widget.description ?? 'Audio',
              style: TextStyle(color: colors.onSurface),
            ),
          ),
          IconButton(
            tooltip: 'Copy audio URL',
            constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
            onPressed: widget.url == null
                ? null
                : () => Clipboard.setData(ClipboardData(text: widget.url!)),
            icon: const Icon(Icons.copy_outlined, size: 18),
          ),
        ],
      ),
    );
  }
}
