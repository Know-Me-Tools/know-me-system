// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'entity.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$EntityRecord {

 String get id; String get entityType; String get dataJson;
/// Create a copy of EntityRecord
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EntityRecordCopyWith<EntityRecord> get copyWith => _$EntityRecordCopyWithImpl<EntityRecord>(this as EntityRecord, _$identity);

  /// Serializes this EntityRecord to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EntityRecord&&(identical(other.id, id) || other.id == id)&&(identical(other.entityType, entityType) || other.entityType == entityType)&&(identical(other.dataJson, dataJson) || other.dataJson == dataJson));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,entityType,dataJson);

@override
String toString() {
  return 'EntityRecord(id: $id, entityType: $entityType, dataJson: $dataJson)';
}


}

/// @nodoc
abstract mixin class $EntityRecordCopyWith<$Res>  {
  factory $EntityRecordCopyWith(EntityRecord value, $Res Function(EntityRecord) _then) = _$EntityRecordCopyWithImpl;
@useResult
$Res call({
 String id, String entityType, String dataJson
});




}
/// @nodoc
class _$EntityRecordCopyWithImpl<$Res>
    implements $EntityRecordCopyWith<$Res> {
  _$EntityRecordCopyWithImpl(this._self, this._then);

  final EntityRecord _self;
  final $Res Function(EntityRecord) _then;

/// Create a copy of EntityRecord
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? entityType = null,Object? dataJson = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,dataJson: null == dataJson ? _self.dataJson : dataJson // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [EntityRecord].
extension EntityRecordPatterns on EntityRecord {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _EntityRecord value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _EntityRecord() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _EntityRecord value)  $default,){
final _that = this;
switch (_that) {
case _EntityRecord():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _EntityRecord value)?  $default,){
final _that = this;
switch (_that) {
case _EntityRecord() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String entityType,  String dataJson)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _EntityRecord() when $default != null:
return $default(_that.id,_that.entityType,_that.dataJson);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String entityType,  String dataJson)  $default,) {final _that = this;
switch (_that) {
case _EntityRecord():
return $default(_that.id,_that.entityType,_that.dataJson);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String entityType,  String dataJson)?  $default,) {final _that = this;
switch (_that) {
case _EntityRecord() when $default != null:
return $default(_that.id,_that.entityType,_that.dataJson);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _EntityRecord implements EntityRecord {
  const _EntityRecord({required this.id, required this.entityType, required this.dataJson});
  factory _EntityRecord.fromJson(Map<String, dynamic> json) => _$EntityRecordFromJson(json);

@override final  String id;
@override final  String entityType;
@override final  String dataJson;

/// Create a copy of EntityRecord
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$EntityRecordCopyWith<_EntityRecord> get copyWith => __$EntityRecordCopyWithImpl<_EntityRecord>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$EntityRecordToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _EntityRecord&&(identical(other.id, id) || other.id == id)&&(identical(other.entityType, entityType) || other.entityType == entityType)&&(identical(other.dataJson, dataJson) || other.dataJson == dataJson));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,id,entityType,dataJson);

@override
String toString() {
  return 'EntityRecord(id: $id, entityType: $entityType, dataJson: $dataJson)';
}


}

/// @nodoc
abstract mixin class _$EntityRecordCopyWith<$Res> implements $EntityRecordCopyWith<$Res> {
  factory _$EntityRecordCopyWith(_EntityRecord value, $Res Function(_EntityRecord) _then) = __$EntityRecordCopyWithImpl;
@override @useResult
$Res call({
 String id, String entityType, String dataJson
});




}
/// @nodoc
class __$EntityRecordCopyWithImpl<$Res>
    implements _$EntityRecordCopyWith<$Res> {
  __$EntityRecordCopyWithImpl(this._self, this._then);

  final _EntityRecord _self;
  final $Res Function(_EntityRecord) _then;

/// Create a copy of EntityRecord
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? entityType = null,Object? dataJson = null,}) {
  return _then(_EntityRecord(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,dataJson: null == dataJson ? _self.dataJson : dataJson // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}


/// @nodoc
mixin _$ListResult {

 List<EntityRecord> get items; String? get nextCursor;
/// Create a copy of ListResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListResultCopyWith<ListResult> get copyWith => _$ListResultCopyWithImpl<ListResult>(this as ListResult, _$identity);

  /// Serializes this ListResult to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListResult&&const DeepCollectionEquality().equals(other.items, items)&&(identical(other.nextCursor, nextCursor) || other.nextCursor == nextCursor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(items),nextCursor);

@override
String toString() {
  return 'ListResult(items: $items, nextCursor: $nextCursor)';
}


}

/// @nodoc
abstract mixin class $ListResultCopyWith<$Res>  {
  factory $ListResultCopyWith(ListResult value, $Res Function(ListResult) _then) = _$ListResultCopyWithImpl;
@useResult
$Res call({
 List<EntityRecord> items, String? nextCursor
});




}
/// @nodoc
class _$ListResultCopyWithImpl<$Res>
    implements $ListResultCopyWith<$Res> {
  _$ListResultCopyWithImpl(this._self, this._then);

  final ListResult _self;
  final $Res Function(ListResult) _then;

/// Create a copy of ListResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? items = null,Object? nextCursor = freezed,}) {
  return _then(_self.copyWith(
items: null == items ? _self.items : items // ignore: cast_nullable_to_non_nullable
as List<EntityRecord>,nextCursor: freezed == nextCursor ? _self.nextCursor : nextCursor // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [ListResult].
extension ListResultPatterns on ListResult {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ListResult value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ListResult() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ListResult value)  $default,){
final _that = this;
switch (_that) {
case _ListResult():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ListResult value)?  $default,){
final _that = this;
switch (_that) {
case _ListResult() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<EntityRecord> items,  String? nextCursor)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ListResult() when $default != null:
return $default(_that.items,_that.nextCursor);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<EntityRecord> items,  String? nextCursor)  $default,) {final _that = this;
switch (_that) {
case _ListResult():
return $default(_that.items,_that.nextCursor);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<EntityRecord> items,  String? nextCursor)?  $default,) {final _that = this;
switch (_that) {
case _ListResult() when $default != null:
return $default(_that.items,_that.nextCursor);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ListResult implements ListResult {
  const _ListResult({final  List<EntityRecord> items = const <EntityRecord>[], this.nextCursor}): _items = items;
  factory _ListResult.fromJson(Map<String, dynamic> json) => _$ListResultFromJson(json);

 final  List<EntityRecord> _items;
@override@JsonKey() List<EntityRecord> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}

@override final  String? nextCursor;

/// Create a copy of ListResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ListResultCopyWith<_ListResult> get copyWith => __$ListResultCopyWithImpl<_ListResult>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ListResultToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ListResult&&const DeepCollectionEquality().equals(other._items, _items)&&(identical(other.nextCursor, nextCursor) || other.nextCursor == nextCursor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_items),nextCursor);

@override
String toString() {
  return 'ListResult(items: $items, nextCursor: $nextCursor)';
}


}

/// @nodoc
abstract mixin class _$ListResultCopyWith<$Res> implements $ListResultCopyWith<$Res> {
  factory _$ListResultCopyWith(_ListResult value, $Res Function(_ListResult) _then) = __$ListResultCopyWithImpl;
@override @useResult
$Res call({
 List<EntityRecord> items, String? nextCursor
});




}
/// @nodoc
class __$ListResultCopyWithImpl<$Res>
    implements _$ListResultCopyWith<$Res> {
  __$ListResultCopyWithImpl(this._self, this._then);

  final _ListResult _self;
  final $Res Function(_ListResult) _then;

/// Create a copy of ListResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? items = null,Object? nextCursor = freezed,}) {
  return _then(_ListResult(
items: null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<EntityRecord>,nextCursor: freezed == nextCursor ? _self.nextCursor : nextCursor // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

ChangeEvent _$ChangeEventFromJson(
  Map<String, dynamic> json
) {
        switch (json['op']) {
                  case 'upsert':
          return ChangeUpsert.fromJson(
            json
          );
                case 'delete':
          return ChangeDelete.fromJson(
            json
          );
                case 'invalidate':
          return ChangeInvalidate.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'op',
  'ChangeEvent',
  'Invalid union type "${json['op']}"!'
);
        }
      
}

/// @nodoc
mixin _$ChangeEvent {



  /// Serializes this ChangeEvent to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChangeEvent);
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ChangeEvent()';
}


}

/// @nodoc
class $ChangeEventCopyWith<$Res>  {
$ChangeEventCopyWith(ChangeEvent _, $Res Function(ChangeEvent) __);
}


/// Adds pattern-matching-related methods to [ChangeEvent].
extension ChangeEventPatterns on ChangeEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ChangeUpsert value)?  upsert,TResult Function( ChangeDelete value)?  delete,TResult Function( ChangeInvalidate value)?  invalidate,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ChangeUpsert() when upsert != null:
return upsert(_that);case ChangeDelete() when delete != null:
return delete(_that);case ChangeInvalidate() when invalidate != null:
return invalidate(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ChangeUpsert value)  upsert,required TResult Function( ChangeDelete value)  delete,required TResult Function( ChangeInvalidate value)  invalidate,}){
final _that = this;
switch (_that) {
case ChangeUpsert():
return upsert(_that);case ChangeDelete():
return delete(_that);case ChangeInvalidate():
return invalidate(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ChangeUpsert value)?  upsert,TResult? Function( ChangeDelete value)?  delete,TResult? Function( ChangeInvalidate value)?  invalidate,}){
final _that = this;
switch (_that) {
case ChangeUpsert() when upsert != null:
return upsert(_that);case ChangeDelete() when delete != null:
return delete(_that);case ChangeInvalidate() when invalidate != null:
return invalidate(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( EntityRecord record)?  upsert,TResult Function( String entityType,  String id)?  delete,TResult Function( String entityType,  String? listKey)?  invalidate,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ChangeUpsert() when upsert != null:
return upsert(_that.record);case ChangeDelete() when delete != null:
return delete(_that.entityType,_that.id);case ChangeInvalidate() when invalidate != null:
return invalidate(_that.entityType,_that.listKey);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( EntityRecord record)  upsert,required TResult Function( String entityType,  String id)  delete,required TResult Function( String entityType,  String? listKey)  invalidate,}) {final _that = this;
switch (_that) {
case ChangeUpsert():
return upsert(_that.record);case ChangeDelete():
return delete(_that.entityType,_that.id);case ChangeInvalidate():
return invalidate(_that.entityType,_that.listKey);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( EntityRecord record)?  upsert,TResult? Function( String entityType,  String id)?  delete,TResult? Function( String entityType,  String? listKey)?  invalidate,}) {final _that = this;
switch (_that) {
case ChangeUpsert() when upsert != null:
return upsert(_that.record);case ChangeDelete() when delete != null:
return delete(_that.entityType,_that.id);case ChangeInvalidate() when invalidate != null:
return invalidate(_that.entityType,_that.listKey);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class ChangeUpsert implements ChangeEvent {
  const ChangeUpsert({required this.record, final  String? $type}): $type = $type ?? 'upsert';
  factory ChangeUpsert.fromJson(Map<String, dynamic> json) => _$ChangeUpsertFromJson(json);

 final  EntityRecord record;

@JsonKey(name: 'op')
final String $type;


/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ChangeUpsertCopyWith<ChangeUpsert> get copyWith => _$ChangeUpsertCopyWithImpl<ChangeUpsert>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ChangeUpsertToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChangeUpsert&&(identical(other.record, record) || other.record == record));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,record);

@override
String toString() {
  return 'ChangeEvent.upsert(record: $record)';
}


}

/// @nodoc
abstract mixin class $ChangeUpsertCopyWith<$Res> implements $ChangeEventCopyWith<$Res> {
  factory $ChangeUpsertCopyWith(ChangeUpsert value, $Res Function(ChangeUpsert) _then) = _$ChangeUpsertCopyWithImpl;
@useResult
$Res call({
 EntityRecord record
});


$EntityRecordCopyWith<$Res> get record;

}
/// @nodoc
class _$ChangeUpsertCopyWithImpl<$Res>
    implements $ChangeUpsertCopyWith<$Res> {
  _$ChangeUpsertCopyWithImpl(this._self, this._then);

  final ChangeUpsert _self;
  final $Res Function(ChangeUpsert) _then;

/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? record = null,}) {
  return _then(ChangeUpsert(
record: null == record ? _self.record : record // ignore: cast_nullable_to_non_nullable
as EntityRecord,
  ));
}

/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$EntityRecordCopyWith<$Res> get record {
  
  return $EntityRecordCopyWith<$Res>(_self.record, (value) {
    return _then(_self.copyWith(record: value));
  });
}
}

/// @nodoc
@JsonSerializable()

class ChangeDelete implements ChangeEvent {
  const ChangeDelete({required this.entityType, required this.id, final  String? $type}): $type = $type ?? 'delete';
  factory ChangeDelete.fromJson(Map<String, dynamic> json) => _$ChangeDeleteFromJson(json);

 final  String entityType;
 final  String id;

@JsonKey(name: 'op')
final String $type;


/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ChangeDeleteCopyWith<ChangeDelete> get copyWith => _$ChangeDeleteCopyWithImpl<ChangeDelete>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ChangeDeleteToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChangeDelete&&(identical(other.entityType, entityType) || other.entityType == entityType)&&(identical(other.id, id) || other.id == id));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,entityType,id);

@override
String toString() {
  return 'ChangeEvent.delete(entityType: $entityType, id: $id)';
}


}

/// @nodoc
abstract mixin class $ChangeDeleteCopyWith<$Res> implements $ChangeEventCopyWith<$Res> {
  factory $ChangeDeleteCopyWith(ChangeDelete value, $Res Function(ChangeDelete) _then) = _$ChangeDeleteCopyWithImpl;
@useResult
$Res call({
 String entityType, String id
});




}
/// @nodoc
class _$ChangeDeleteCopyWithImpl<$Res>
    implements $ChangeDeleteCopyWith<$Res> {
  _$ChangeDeleteCopyWithImpl(this._self, this._then);

  final ChangeDelete _self;
  final $Res Function(ChangeDelete) _then;

/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? entityType = null,Object? id = null,}) {
  return _then(ChangeDelete(
entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
@JsonSerializable()

class ChangeInvalidate implements ChangeEvent {
  const ChangeInvalidate({required this.entityType, this.listKey, final  String? $type}): $type = $type ?? 'invalidate';
  factory ChangeInvalidate.fromJson(Map<String, dynamic> json) => _$ChangeInvalidateFromJson(json);

 final  String entityType;
 final  String? listKey;

@JsonKey(name: 'op')
final String $type;


/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ChangeInvalidateCopyWith<ChangeInvalidate> get copyWith => _$ChangeInvalidateCopyWithImpl<ChangeInvalidate>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ChangeInvalidateToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChangeInvalidate&&(identical(other.entityType, entityType) || other.entityType == entityType)&&(identical(other.listKey, listKey) || other.listKey == listKey));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,entityType,listKey);

@override
String toString() {
  return 'ChangeEvent.invalidate(entityType: $entityType, listKey: $listKey)';
}


}

/// @nodoc
abstract mixin class $ChangeInvalidateCopyWith<$Res> implements $ChangeEventCopyWith<$Res> {
  factory $ChangeInvalidateCopyWith(ChangeInvalidate value, $Res Function(ChangeInvalidate) _then) = _$ChangeInvalidateCopyWithImpl;
@useResult
$Res call({
 String entityType, String? listKey
});




}
/// @nodoc
class _$ChangeInvalidateCopyWithImpl<$Res>
    implements $ChangeInvalidateCopyWith<$Res> {
  _$ChangeInvalidateCopyWithImpl(this._self, this._then);

  final ChangeInvalidate _self;
  final $Res Function(ChangeInvalidate) _then;

/// Create a copy of ChangeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? entityType = null,Object? listKey = freezed,}) {
  return _then(ChangeInvalidate(
entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,listKey: freezed == listKey ? _self.listKey : listKey // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
