// TJ-ARCH-MOB-001 compliant
//
// Pins the conversation-policy shape contract across the FFI boundary.
//
// Rust returns the **bare** `RunPolicy` — `chat_mode` at the top level — and
// `save_conversation_policy` accepts that same shape. It used to return the
// storage wrapper (`{conversation_id, policy, updated_at}`) instead, which broke
// the mode picker silently on device: the notifier read `policy['chat_mode']`
// (absent, so the label stayed on the default) and then sent the wrapper back to
// save, where every `RunPolicy` field being `#[serde(default)]` meant it
// deserialized to defaults *without erroring* and quietly discarded the
// selection. No exception, no log line, no visible change.
import 'package:flutter_test/flutter_test.dart';

import 'package:knowme_poc/features/chat/presentation/providers/chat_control_notifier.dart';

void main() {
  test('a bare stored policy overrides the defaults', () {
    final policy = mergeConversationPolicy({
      'version': 1,
      'chat_mode': 'local',
      'agent_id': null,
    });

    expect(policy['chat_mode'], 'local');
    expect(policy['agent_id'], isNull);
    // Fields the stored policy omits still come from the defaults, so what goes
    // back to `savePolicy` is a complete RunPolicy rather than a sparse patch.
    expect(policy['tool_approval'], 'ask');
    expect(policy['skills'], isA<Map<String, dynamic>>());
  });

  test('a null policy yields the defaults', () {
    // A brand-new conversation has nothing saved; Rust returns null.
    final policy = mergeConversationPolicy(null);

    expect(policy['chat_mode'], 'uar');
    expect(policy['agent_id'], 'orchestrator-agent');
  });

  test('the storage wrapper shape does not satisfy the contract', () {
    // If the accessor ever leaks ConversationPolicyRecord again, chat_mode is
    // nested and the top-level read misses it. Assert that explicitly so the
    // regression fails here instead of shipping as a dead button.
    final policy = mergeConversationPolicy({
      'conversation_id': 'c1',
      'policy': {'chat_mode': 'local'},
      'updated_at': '2026-07-26T00:00:00Z',
    });

    expect(
      policy['chat_mode'],
      isNot('local'),
      reason: 'nested wrapper must not read as a selected mode',
    );
  });
}
