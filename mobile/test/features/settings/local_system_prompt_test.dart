// TJ-ARCH-MOB-001 compliant
//
// Contract tests for the local system prompt.
//
// The prompt is stored in the local settings database and resolved by Rust: a
// blank stored value falls back to the shipped default, because an empty system
// prompt would strip the assistant's framing entirely and the context assembler
// requires a leading system message. These tests pin that "clear means reset"
// round-trip through the repository seam, which is where the UI's reset
// affordance depends on reading back the *effective* value rather than what was
// typed.
import 'package:flutter_test/flutter_test.dart';

import 'package:knowme_poc/features/settings/domain/settings_repository.dart';
import 'package:knowme_poc/features/settings/domain/settings_snapshot.dart';

const _shipped = 'You are KnowMe, a helpful private assistant.';

void main() {
  test('a saved prompt is read back verbatim', () async {
    final repository = _FakeSettingsRepository();

    await repository.saveLocalSystemPrompt('Answer only in haiku.');

    expect(await repository.localSystemPrompt(), 'Answer only in haiku.');
    expect(
      (await repository.load()).localSystemPrompt,
      'Answer only in haiku.',
    );
  });

  test('clearing the prompt restores the shipped default', () async {
    final repository = _FakeSettingsRepository();
    await repository.saveLocalSystemPrompt('Answer only in haiku.');

    // The editor's "leave blank and save" path. Reading back must yield the
    // default, not an empty string — an empty system prompt is not a valid
    // state for the context assembler.
    await repository.saveLocalSystemPrompt('   ');

    expect(await repository.localSystemPrompt(), _shipped);
  });

  test('the reset affordance sources its copy from Rust', () async {
    final repository = _FakeSettingsRepository();

    // Reset writes the shipped default rather than a Dart-side literal, so the
    // two cannot drift apart.
    final shipped = await repository.defaultLocalSystemPrompt();
    await repository.saveLocalSystemPrompt(shipped);

    expect(shipped, _shipped);
    expect(await repository.localSystemPrompt(), _shipped);
  });

  test('an unset prompt reports the default', () async {
    expect(await _FakeSettingsRepository().localSystemPrompt(), _shipped);
  });
}

/// Mirrors the Rust resolution rule: a blank stored override means "use the
/// default", so the fake resolves rather than echoing what it was handed.
class _FakeSettingsRepository implements SettingsRepository {
  String? _stored;

  @override
  Future<void> saveLocalSystemPrompt(String prompt) async {
    final trimmed = prompt.trim();
    _stored = trimmed.isEmpty ? null : trimmed;
  }

  @override
  Future<String> localSystemPrompt() async => _stored ?? _shipped;

  @override
  Future<String> defaultLocalSystemPrompt() async => _shipped;

  @override
  Future<SettingsSnapshot> load() async =>
      SettingsSnapshot(localSystemPrompt: await localSystemPrompt());

  @override
  Future<String> errorMessage(Object error) async => error.toString();

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('not needed by these tests');
}
