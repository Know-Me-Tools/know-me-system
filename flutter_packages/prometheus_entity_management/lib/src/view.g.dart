// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'view.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_FilterSpec _$FilterSpecFromJson(Map<String, dynamic> json) => _FilterSpec(
  field: json['field'] as String,
  op: $enumDecode(_$FilterOpEnumMap, json['op']),
  valueJson: json['valueJson'] as String,
);

Map<String, dynamic> _$FilterSpecToJson(_FilterSpec instance) =>
    <String, dynamic>{
      'field': instance.field,
      'op': _$FilterOpEnumMap[instance.op]!,
      'valueJson': instance.valueJson,
    };

const _$FilterOpEnumMap = {
  FilterOp.eq: 'eq',
  FilterOp.ne: 'ne',
  FilterOp.lt: 'lt',
  FilterOp.lte: 'lte',
  FilterOp.gt: 'gt',
  FilterOp.gte: 'gte',
  FilterOp.inList: 'in',
  FilterOp.like: 'like',
};

_SortSpec _$SortSpecFromJson(Map<String, dynamic> json) => _SortSpec(
  field: json['field'] as String,
  descending: json['descending'] as bool? ?? false,
);

Map<String, dynamic> _$SortSpecToJson(_SortSpec instance) => <String, dynamic>{
  'field': instance.field,
  'descending': instance.descending,
};

_ViewDescriptor _$ViewDescriptorFromJson(Map<String, dynamic> json) =>
    _ViewDescriptor(
      entityType: json['entityType'] as String,
      filters:
          (json['filters'] as List<dynamic>?)
              ?.map((e) => FilterSpec.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const <FilterSpec>[],
      sorts:
          (json['sorts'] as List<dynamic>?)
              ?.map((e) => SortSpec.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const <SortSpec>[],
      limit: (json['limit'] as num?)?.toInt(),
      cursor: json['cursor'] as String?,
    );

Map<String, dynamic> _$ViewDescriptorToJson(_ViewDescriptor instance) =>
    <String, dynamic>{
      'entityType': instance.entityType,
      'filters': instance.filters,
      'sorts': instance.sorts,
      'limit': instance.limit,
      'cursor': instance.cursor,
    };
