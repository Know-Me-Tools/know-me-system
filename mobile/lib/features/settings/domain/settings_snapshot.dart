// TJ-ARCH-MOB-001 compliant
import 'package:flutter/foundation.dart';

@immutable
class SettingsSnapshot {
  const SettingsSnapshot({
    this.activeLane = 'local',
    this.uarConfig = const {},
    this.admin = const {},
    this.catalog = const {},
    this.localModel = const {},
    this.localDiagnostics = const {},
    this.providers = const [],
    this.configuredProviders = const [],
    this.cloudModel,
  });

  final String activeLane;
  final Map<String, dynamic> uarConfig;
  final Map<String, dynamic> admin;
  final Map<String, dynamic> catalog;
  final Map<String, dynamic> localModel;
  final Map<String, dynamic> localDiagnostics;
  final List<Map<String, dynamic>> providers;
  final List<Map<String, dynamic>> configuredProviders;
  final Map<String, dynamic>? cloudModel;

  SettingsSnapshot copyWith({
    String? activeLane,
    Map<String, dynamic>? uarConfig,
    Map<String, dynamic>? admin,
    Map<String, dynamic>? catalog,
    Map<String, dynamic>? localModel,
    Map<String, dynamic>? localDiagnostics,
    List<Map<String, dynamic>>? providers,
    List<Map<String, dynamic>>? configuredProviders,
    Map<String, dynamic>? cloudModel,
  }) => SettingsSnapshot(
    activeLane: activeLane ?? this.activeLane,
    uarConfig: uarConfig ?? this.uarConfig,
    admin: admin ?? this.admin,
    catalog: catalog ?? this.catalog,
    localModel: localModel ?? this.localModel,
    localDiagnostics: localDiagnostics ?? this.localDiagnostics,
    providers: providers ?? this.providers,
    configuredProviders: configuredProviders ?? this.configuredProviders,
    cloudModel: cloudModel ?? this.cloudModel,
  );
}
