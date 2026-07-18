// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'view.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$FilterSpec {

 String get field; FilterOp get op; String get valueJson;
/// Create a copy of FilterSpec
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilterSpecCopyWith<FilterSpec> get copyWith => _$FilterSpecCopyWithImpl<FilterSpec>(this as FilterSpec, _$identity);

  /// Serializes this FilterSpec to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilterSpec&&(identical(other.field, field) || other.field == field)&&(identical(other.op, op) || other.op == op)&&(identical(other.valueJson, valueJson) || other.valueJson == valueJson));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,field,op,valueJson);

@override
String toString() {
  return 'FilterSpec(field: $field, op: $op, valueJson: $valueJson)';
}


}

/// @nodoc
abstract mixin class $FilterSpecCopyWith<$Res>  {
  factory $FilterSpecCopyWith(FilterSpec value, $Res Function(FilterSpec) _then) = _$FilterSpecCopyWithImpl;
@useResult
$Res call({
 String field, FilterOp op, String valueJson
});




}
/// @nodoc
class _$FilterSpecCopyWithImpl<$Res>
    implements $FilterSpecCopyWith<$Res> {
  _$FilterSpecCopyWithImpl(this._self, this._then);

  final FilterSpec _self;
  final $Res Function(FilterSpec) _then;

/// Create a copy of FilterSpec
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? field = null,Object? op = null,Object? valueJson = null,}) {
  return _then(_self.copyWith(
field: null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String,op: null == op ? _self.op : op // ignore: cast_nullable_to_non_nullable
as FilterOp,valueJson: null == valueJson ? _self.valueJson : valueJson // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [FilterSpec].
extension FilterSpecPatterns on FilterSpec {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FilterSpec value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FilterSpec() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FilterSpec value)  $default,){
final _that = this;
switch (_that) {
case _FilterSpec():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FilterSpec value)?  $default,){
final _that = this;
switch (_that) {
case _FilterSpec() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String field,  FilterOp op,  String valueJson)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FilterSpec() when $default != null:
return $default(_that.field,_that.op,_that.valueJson);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String field,  FilterOp op,  String valueJson)  $default,) {final _that = this;
switch (_that) {
case _FilterSpec():
return $default(_that.field,_that.op,_that.valueJson);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String field,  FilterOp op,  String valueJson)?  $default,) {final _that = this;
switch (_that) {
case _FilterSpec() when $default != null:
return $default(_that.field,_that.op,_that.valueJson);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _FilterSpec implements FilterSpec {
  const _FilterSpec({required this.field, required this.op, required this.valueJson});
  factory _FilterSpec.fromJson(Map<String, dynamic> json) => _$FilterSpecFromJson(json);

@override final  String field;
@override final  FilterOp op;
@override final  String valueJson;

/// Create a copy of FilterSpec
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FilterSpecCopyWith<_FilterSpec> get copyWith => __$FilterSpecCopyWithImpl<_FilterSpec>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$FilterSpecToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FilterSpec&&(identical(other.field, field) || other.field == field)&&(identical(other.op, op) || other.op == op)&&(identical(other.valueJson, valueJson) || other.valueJson == valueJson));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,field,op,valueJson);

@override
String toString() {
  return 'FilterSpec(field: $field, op: $op, valueJson: $valueJson)';
}


}

/// @nodoc
abstract mixin class _$FilterSpecCopyWith<$Res> implements $FilterSpecCopyWith<$Res> {
  factory _$FilterSpecCopyWith(_FilterSpec value, $Res Function(_FilterSpec) _then) = __$FilterSpecCopyWithImpl;
@override @useResult
$Res call({
 String field, FilterOp op, String valueJson
});




}
/// @nodoc
class __$FilterSpecCopyWithImpl<$Res>
    implements _$FilterSpecCopyWith<$Res> {
  __$FilterSpecCopyWithImpl(this._self, this._then);

  final _FilterSpec _self;
  final $Res Function(_FilterSpec) _then;

/// Create a copy of FilterSpec
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? field = null,Object? op = null,Object? valueJson = null,}) {
  return _then(_FilterSpec(
field: null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String,op: null == op ? _self.op : op // ignore: cast_nullable_to_non_nullable
as FilterOp,valueJson: null == valueJson ? _self.valueJson : valueJson // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}


/// @nodoc
mixin _$SortSpec {

 String get field; bool get descending;
/// Create a copy of SortSpec
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SortSpecCopyWith<SortSpec> get copyWith => _$SortSpecCopyWithImpl<SortSpec>(this as SortSpec, _$identity);

  /// Serializes this SortSpec to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SortSpec&&(identical(other.field, field) || other.field == field)&&(identical(other.descending, descending) || other.descending == descending));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,field,descending);

@override
String toString() {
  return 'SortSpec(field: $field, descending: $descending)';
}


}

/// @nodoc
abstract mixin class $SortSpecCopyWith<$Res>  {
  factory $SortSpecCopyWith(SortSpec value, $Res Function(SortSpec) _then) = _$SortSpecCopyWithImpl;
@useResult
$Res call({
 String field, bool descending
});




}
/// @nodoc
class _$SortSpecCopyWithImpl<$Res>
    implements $SortSpecCopyWith<$Res> {
  _$SortSpecCopyWithImpl(this._self, this._then);

  final SortSpec _self;
  final $Res Function(SortSpec) _then;

/// Create a copy of SortSpec
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? field = null,Object? descending = null,}) {
  return _then(_self.copyWith(
field: null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [SortSpec].
extension SortSpecPatterns on SortSpec {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SortSpec value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SortSpec() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SortSpec value)  $default,){
final _that = this;
switch (_that) {
case _SortSpec():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SortSpec value)?  $default,){
final _that = this;
switch (_that) {
case _SortSpec() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String field,  bool descending)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SortSpec() when $default != null:
return $default(_that.field,_that.descending);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String field,  bool descending)  $default,) {final _that = this;
switch (_that) {
case _SortSpec():
return $default(_that.field,_that.descending);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String field,  bool descending)?  $default,) {final _that = this;
switch (_that) {
case _SortSpec() when $default != null:
return $default(_that.field,_that.descending);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _SortSpec implements SortSpec {
  const _SortSpec({required this.field, this.descending = false});
  factory _SortSpec.fromJson(Map<String, dynamic> json) => _$SortSpecFromJson(json);

@override final  String field;
@override@JsonKey() final  bool descending;

/// Create a copy of SortSpec
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SortSpecCopyWith<_SortSpec> get copyWith => __$SortSpecCopyWithImpl<_SortSpec>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$SortSpecToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SortSpec&&(identical(other.field, field) || other.field == field)&&(identical(other.descending, descending) || other.descending == descending));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,field,descending);

@override
String toString() {
  return 'SortSpec(field: $field, descending: $descending)';
}


}

/// @nodoc
abstract mixin class _$SortSpecCopyWith<$Res> implements $SortSpecCopyWith<$Res> {
  factory _$SortSpecCopyWith(_SortSpec value, $Res Function(_SortSpec) _then) = __$SortSpecCopyWithImpl;
@override @useResult
$Res call({
 String field, bool descending
});




}
/// @nodoc
class __$SortSpecCopyWithImpl<$Res>
    implements _$SortSpecCopyWith<$Res> {
  __$SortSpecCopyWithImpl(this._self, this._then);

  final _SortSpec _self;
  final $Res Function(_SortSpec) _then;

/// Create a copy of SortSpec
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? field = null,Object? descending = null,}) {
  return _then(_SortSpec(
field: null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}


/// @nodoc
mixin _$ViewDescriptor {

 String get entityType; List<FilterSpec> get filters; List<SortSpec> get sorts; int? get limit; String? get cursor;
/// Create a copy of ViewDescriptor
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ViewDescriptorCopyWith<ViewDescriptor> get copyWith => _$ViewDescriptorCopyWithImpl<ViewDescriptor>(this as ViewDescriptor, _$identity);

  /// Serializes this ViewDescriptor to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ViewDescriptor&&(identical(other.entityType, entityType) || other.entityType == entityType)&&const DeepCollectionEquality().equals(other.filters, filters)&&const DeepCollectionEquality().equals(other.sorts, sorts)&&(identical(other.limit, limit) || other.limit == limit)&&(identical(other.cursor, cursor) || other.cursor == cursor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,entityType,const DeepCollectionEquality().hash(filters),const DeepCollectionEquality().hash(sorts),limit,cursor);

@override
String toString() {
  return 'ViewDescriptor(entityType: $entityType, filters: $filters, sorts: $sorts, limit: $limit, cursor: $cursor)';
}


}

/// @nodoc
abstract mixin class $ViewDescriptorCopyWith<$Res>  {
  factory $ViewDescriptorCopyWith(ViewDescriptor value, $Res Function(ViewDescriptor) _then) = _$ViewDescriptorCopyWithImpl;
@useResult
$Res call({
 String entityType, List<FilterSpec> filters, List<SortSpec> sorts, int? limit, String? cursor
});




}
/// @nodoc
class _$ViewDescriptorCopyWithImpl<$Res>
    implements $ViewDescriptorCopyWith<$Res> {
  _$ViewDescriptorCopyWithImpl(this._self, this._then);

  final ViewDescriptor _self;
  final $Res Function(ViewDescriptor) _then;

/// Create a copy of ViewDescriptor
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? entityType = null,Object? filters = null,Object? sorts = null,Object? limit = freezed,Object? cursor = freezed,}) {
  return _then(_self.copyWith(
entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,filters: null == filters ? _self.filters : filters // ignore: cast_nullable_to_non_nullable
as List<FilterSpec>,sorts: null == sorts ? _self.sorts : sorts // ignore: cast_nullable_to_non_nullable
as List<SortSpec>,limit: freezed == limit ? _self.limit : limit // ignore: cast_nullable_to_non_nullable
as int?,cursor: freezed == cursor ? _self.cursor : cursor // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [ViewDescriptor].
extension ViewDescriptorPatterns on ViewDescriptor {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ViewDescriptor value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ViewDescriptor() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ViewDescriptor value)  $default,){
final _that = this;
switch (_that) {
case _ViewDescriptor():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ViewDescriptor value)?  $default,){
final _that = this;
switch (_that) {
case _ViewDescriptor() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String entityType,  List<FilterSpec> filters,  List<SortSpec> sorts,  int? limit,  String? cursor)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ViewDescriptor() when $default != null:
return $default(_that.entityType,_that.filters,_that.sorts,_that.limit,_that.cursor);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String entityType,  List<FilterSpec> filters,  List<SortSpec> sorts,  int? limit,  String? cursor)  $default,) {final _that = this;
switch (_that) {
case _ViewDescriptor():
return $default(_that.entityType,_that.filters,_that.sorts,_that.limit,_that.cursor);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String entityType,  List<FilterSpec> filters,  List<SortSpec> sorts,  int? limit,  String? cursor)?  $default,) {final _that = this;
switch (_that) {
case _ViewDescriptor() when $default != null:
return $default(_that.entityType,_that.filters,_that.sorts,_that.limit,_that.cursor);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ViewDescriptor implements ViewDescriptor {
  const _ViewDescriptor({required this.entityType, final  List<FilterSpec> filters = const <FilterSpec>[], final  List<SortSpec> sorts = const <SortSpec>[], this.limit, this.cursor}): _filters = filters,_sorts = sorts;
  factory _ViewDescriptor.fromJson(Map<String, dynamic> json) => _$ViewDescriptorFromJson(json);

@override final  String entityType;
 final  List<FilterSpec> _filters;
@override@JsonKey() List<FilterSpec> get filters {
  if (_filters is EqualUnmodifiableListView) return _filters;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_filters);
}

 final  List<SortSpec> _sorts;
@override@JsonKey() List<SortSpec> get sorts {
  if (_sorts is EqualUnmodifiableListView) return _sorts;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_sorts);
}

@override final  int? limit;
@override final  String? cursor;

/// Create a copy of ViewDescriptor
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ViewDescriptorCopyWith<_ViewDescriptor> get copyWith => __$ViewDescriptorCopyWithImpl<_ViewDescriptor>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ViewDescriptorToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ViewDescriptor&&(identical(other.entityType, entityType) || other.entityType == entityType)&&const DeepCollectionEquality().equals(other._filters, _filters)&&const DeepCollectionEquality().equals(other._sorts, _sorts)&&(identical(other.limit, limit) || other.limit == limit)&&(identical(other.cursor, cursor) || other.cursor == cursor));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,entityType,const DeepCollectionEquality().hash(_filters),const DeepCollectionEquality().hash(_sorts),limit,cursor);

@override
String toString() {
  return 'ViewDescriptor(entityType: $entityType, filters: $filters, sorts: $sorts, limit: $limit, cursor: $cursor)';
}


}

/// @nodoc
abstract mixin class _$ViewDescriptorCopyWith<$Res> implements $ViewDescriptorCopyWith<$Res> {
  factory _$ViewDescriptorCopyWith(_ViewDescriptor value, $Res Function(_ViewDescriptor) _then) = __$ViewDescriptorCopyWithImpl;
@override @useResult
$Res call({
 String entityType, List<FilterSpec> filters, List<SortSpec> sorts, int? limit, String? cursor
});




}
/// @nodoc
class __$ViewDescriptorCopyWithImpl<$Res>
    implements _$ViewDescriptorCopyWith<$Res> {
  __$ViewDescriptorCopyWithImpl(this._self, this._then);

  final _ViewDescriptor _self;
  final $Res Function(_ViewDescriptor) _then;

/// Create a copy of ViewDescriptor
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? entityType = null,Object? filters = null,Object? sorts = null,Object? limit = freezed,Object? cursor = freezed,}) {
  return _then(_ViewDescriptor(
entityType: null == entityType ? _self.entityType : entityType // ignore: cast_nullable_to_non_nullable
as String,filters: null == filters ? _self._filters : filters // ignore: cast_nullable_to_non_nullable
as List<FilterSpec>,sorts: null == sorts ? _self._sorts : sorts // ignore: cast_nullable_to_non_nullable
as List<SortSpec>,limit: freezed == limit ? _self.limit : limit // ignore: cast_nullable_to_non_nullable
as int?,cursor: freezed == cursor ? _self.cursor : cursor // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
