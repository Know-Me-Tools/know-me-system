# Platform support policy

KnowMe prioritizes high-impact native capabilities on current-generation phones
over legacy device coverage. Minimum toolchain and OS versions may be raised
whenever a current stable dependency, security control, accessibility feature,
media capability, or materially better UX requires it.

Current baseline:

- Dart: 3.13.0-0 or newer
- Flutter: 3.47.0-0.1.pre or newer
- Android: API 29 (Android 10) or newer
- Android native ABI: arm64-v8a
- iOS: 15.0 or newer

Every baseline change must update the manifests, lockfiles, project guidance,
and this note. Before release, verify analysis/tests and install the application
on a current physical Android device; record the Flutter/Dart versions, Android
API level, device model, and verification date in the relevant change record.

## Latest physical-device verification

- Date: 2026-07-18
- Device: Samsung SM-S936U, Android API 36, arm64-v8a
- Toolchain: Flutter 3.47.0-0.1.pre (beta), Dart 3.13.0-282.1.beta
- Result: debug APK built, installed, and launched successfully; WCAG light and
  dark chat palettes, 80% message width, copy and upload controls, theme
  switching, and theme persistence across a cold process restart were visually
  verified on the device.
