// TJ-ARCH-MOB-001 compliant
//
// Proves the DEVICE is offline, from the device.
//
// WHY THIS EXISTS INSTEAD OF A HOST-SIDE CHECK
//
// The obvious place for an offline check is the shell gate that launches the
// suite. It cannot live there: `xcrun devicectl device info details` reports
// hardware and boot properties only — no radio state — and the device's
// `.coredevice.local` name does not resolve. A host-side grep for "wifi
// connected" therefore always misses, and a gate built on it prints
// "OFFLINE VERIFIED" while the phone is fully online.
//
// That is not hypothetical. The Android suite shipped exactly that bug:
// `svc wifi disable` left cellular up, so gates that believed they ran offline
// ran online, and their passes meant nothing.
//
// An attempted connection that FAILS is evidence. Inference from a command
// that cannot see radios is not.
import 'dart:io';

/// Hosts to attempt. Two, on different networks, because a single unreachable
/// host proves only that one host is unreachable — DNS blackholing or a captive
/// portal could produce that while the device is otherwise online.
const _probeHosts = <String>['example.com', 'cloudflare.com'];

/// Short: this must not add minutes to every suite. A device with a live radio
/// answers well inside this; one in airplane mode fails immediately with
/// SocketException rather than timing out.
const _probeTimeout = Duration(seconds: 5);

/// Throws unless every probe fails, i.e. unless the device is genuinely offline.
///
/// Deliberately fails LOUD rather than skipping. A suite that quietly ran
/// online would certify the opposite of what task 5.5 asks, and would look
/// identical to a real pass in the report.
Future<void> assertDeviceIsOffline() async {
  final reachable = <String>[];

  for (final host in _probeHosts) {
    try {
      final socket = await Socket.connect(host, 443, timeout: _probeTimeout);
      socket.destroy();
      reachable.add(host);
    } on SocketException {
      // Expected when offline — this is the passing path.
    } on Exception {
      // Any other failure still means no usable connection was established.
    }
  }

  if (reachable.isNotEmpty) {
    throw StateError(
      'REFUSING TO CERTIFY: the device reached ${reachable.join(", ")}.\n'
      'Task 5.5 requires networking DISABLED. Enable airplane mode on the '
      'device (leave USB attached) and re-run.\n'
      'Wi-Fi alone is not enough — cellular keeps a device online, which is '
      'how the Android suite once certified "offline" runs that were online.',
    );
  }
}
