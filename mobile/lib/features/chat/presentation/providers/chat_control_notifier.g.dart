// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'chat_control_notifier.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(ChatControl)
final chatControlProvider = ChatControlFamily._();

final class ChatControlProvider
    extends $NotifierProvider<ChatControl, ChatControlState> {
  ChatControlProvider._({
    required ChatControlFamily super.from,
    required String super.argument,
  }) : super(
         retry: null,
         name: r'chatControlProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$chatControlHash();

  @override
  String toString() {
    return r'chatControlProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  ChatControl create() => ChatControl();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ChatControlState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ChatControlState>(value),
    );
  }

  @override
  bool operator ==(Object other) {
    return other is ChatControlProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$chatControlHash() => r'31f7f8531e84d4214cea9ecef163ceb9d5f5e583';

final class ChatControlFamily extends $Family
    with
        $ClassFamilyOverride<
          ChatControl,
          ChatControlState,
          ChatControlState,
          ChatControlState,
          String
        > {
  ChatControlFamily._()
    : super(
        retry: null,
        name: r'chatControlProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  ChatControlProvider call(String conversationId) =>
      ChatControlProvider._(argument: conversationId, from: this);

  @override
  String toString() => r'chatControlProvider';
}

abstract class _$ChatControl extends $Notifier<ChatControlState> {
  late final _$args = ref.$arg as String;
  String get conversationId => _$args;

  ChatControlState build(String conversationId);
  @$mustCallSuper
  @override
  void runBuild() {
    final ref = this.ref as $Ref<ChatControlState, ChatControlState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<ChatControlState, ChatControlState>,
              ChatControlState,
              Object?,
              Object?
            >;
    element.handleCreate(ref, () => build(_$args));
  }
}
