// TJ-ARCH-MOB-001 compliant
/// Entity wire types — Dart mirror of gen_ui_types::transport. `data_json` holds
/// the entity payload as JSON so the transport stays schema-agnostic; features
/// decode it into their own freezed models. ChangeEvent is the Rust-emitted
/// invalidation signal the bridge folds into `ref.invalidate`.
import 'package:freezed_annotation/freezed_annotation.dart';

part 'entity.freezed.dart';
part 'entity.g.dart';

@freezed
abstract class EntityRecord with _$EntityRecord {
  const factory EntityRecord({
    required String id,
    required String entityType,
    required String dataJson,
  }) = _EntityRecord;
  factory EntityRecord.fromJson(Map<String, dynamic> json) =>
      _$EntityRecordFromJson(json);
}

@freezed
abstract class ListResult with _$ListResult {
  const factory ListResult({
    @Default(<EntityRecord>[]) List<EntityRecord> items,
    String? nextCursor,
  }) = _ListResult;
  factory ListResult.fromJson(Map<String, dynamic> json) =>
      _$ListResultFromJson(json);
}

/// Mirrors gen_ui_types::transport::ChangeEvent (serde tag = "op", snake_case).
@Freezed(unionKey: 'op', unionValueCase: FreezedUnionCase.snake)
sealed class ChangeEvent with _$ChangeEvent {
  const factory ChangeEvent.upsert({required EntityRecord record}) =
      ChangeUpsert;
  const factory ChangeEvent.delete({
    required String entityType,
    required String id,
  }) = ChangeDelete;
  const factory ChangeEvent.invalidate({
    required String entityType,
    String? listKey,
  }) = ChangeInvalidate;
  factory ChangeEvent.fromJson(Map<String, dynamic> json) =>
      _$ChangeEventFromJson(json);
}
