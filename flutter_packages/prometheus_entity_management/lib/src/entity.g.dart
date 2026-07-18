// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'entity.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_EntityRecord _$EntityRecordFromJson(Map<String, dynamic> json) =>
    _EntityRecord(
      id: json['id'] as String,
      entityType: json['entityType'] as String,
      dataJson: json['dataJson'] as String,
    );

Map<String, dynamic> _$EntityRecordToJson(_EntityRecord instance) =>
    <String, dynamic>{
      'id': instance.id,
      'entityType': instance.entityType,
      'dataJson': instance.dataJson,
    };

_ListResult _$ListResultFromJson(Map<String, dynamic> json) => _ListResult(
  items:
      (json['items'] as List<dynamic>?)
          ?.map((e) => EntityRecord.fromJson(e as Map<String, dynamic>))
          .toList() ??
      const <EntityRecord>[],
  nextCursor: json['nextCursor'] as String?,
);

Map<String, dynamic> _$ListResultToJson(_ListResult instance) =>
    <String, dynamic>{
      'items': instance.items,
      'nextCursor': instance.nextCursor,
    };

ChangeUpsert _$ChangeUpsertFromJson(Map<String, dynamic> json) => ChangeUpsert(
  record: EntityRecord.fromJson(json['record'] as Map<String, dynamic>),
  $type: json['op'] as String?,
);

Map<String, dynamic> _$ChangeUpsertToJson(ChangeUpsert instance) =>
    <String, dynamic>{'record': instance.record, 'op': instance.$type};

ChangeDelete _$ChangeDeleteFromJson(Map<String, dynamic> json) => ChangeDelete(
  entityType: json['entityType'] as String,
  id: json['id'] as String,
  $type: json['op'] as String?,
);

Map<String, dynamic> _$ChangeDeleteToJson(ChangeDelete instance) =>
    <String, dynamic>{
      'entityType': instance.entityType,
      'id': instance.id,
      'op': instance.$type,
    };

ChangeInvalidate _$ChangeInvalidateFromJson(Map<String, dynamic> json) =>
    ChangeInvalidate(
      entityType: json['entityType'] as String,
      listKey: json['listKey'] as String?,
      $type: json['op'] as String?,
    );

Map<String, dynamic> _$ChangeInvalidateToJson(ChangeInvalidate instance) =>
    <String, dynamic>{
      'entityType': instance.entityType,
      'listKey': instance.listKey,
      'op': instance.$type,
    };
