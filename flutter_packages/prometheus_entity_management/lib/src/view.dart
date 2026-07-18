// TJ-ARCH-MOB-001 compliant
/// Transport-agnostic query description — the Dart mirror of
/// gen_ui_types::view (ViewDescriptor / FilterSpec / SortSpec). Compiled to SQL
/// clauses in Rust (gen_ui_db); the UI only ever constructs these. JSON is
/// snake_case to match the Rust serde wire format 1:1.
import 'package:freezed_annotation/freezed_annotation.dart';

part 'view.freezed.dart';
part 'view.g.dart';

enum FilterOp {
  @JsonValue('eq') eq,
  @JsonValue('ne') ne,
  @JsonValue('lt') lt,
  @JsonValue('lte') lte,
  @JsonValue('gt') gt,
  @JsonValue('gte') gte,
  @JsonValue('in') inList,
  @JsonValue('like') like,
}

@freezed
abstract class FilterSpec with _$FilterSpec {
  const factory FilterSpec({
    required String field,
    required FilterOp op,
    required String valueJson,
  }) = _FilterSpec;
  factory FilterSpec.fromJson(Map<String, dynamic> json) =>
      _$FilterSpecFromJson(json);
}

@freezed
abstract class SortSpec with _$SortSpec {
  const factory SortSpec({
    required String field,
    @Default(false) bool descending,
  }) = _SortSpec;
  factory SortSpec.fromJson(Map<String, dynamic> json) =>
      _$SortSpecFromJson(json);
}

@freezed
abstract class ViewDescriptor with _$ViewDescriptor {
  const factory ViewDescriptor({
    required String entityType,
    @Default(<FilterSpec>[]) List<FilterSpec> filters,
    @Default(<SortSpec>[]) List<SortSpec> sorts,
    int? limit,
    String? cursor,
  }) = _ViewDescriptor;
  factory ViewDescriptor.fromJson(Map<String, dynamic> json) =>
      _$ViewDescriptorFromJson(json);
}
