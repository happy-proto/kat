; Repository-local SQL highlights query.
; Based on the upstream tree-sitter-sql structure, but trimmed to the subset of
; node types that is stable in the generated parser we build in-repo.

(invocation
  (object_reference
    name: (identifier) @function.call))

(object_reference
  name: (identifier) @type)

(relation
  alias: (identifier) @variable)

(term
  alias: (identifier) @variable)

(statement
  alias: (identifier) @field)

(field
  name: (identifier) @field)

(parameter) @parameter

[
  (comment)
  (marginalia)
] @comment

(literal) @string

((literal) @number
  (#match? @number "^[-+]?[0-9]+$"))

((literal) @float
  (#match? @float "^[-+]?[0-9]*\\.[0-9]+$"))

[
  (keyword_true)
  (keyword_false)
] @boolean

[
  (keyword_case)
  (keyword_else)
  (keyword_end)
  (keyword_then)
  (keyword_when)
] @conditional

[
  (keyword_and)
  (keyword_between)
  (keyword_exists)
  (keyword_in)
  (keyword_is)
  (keyword_like)
  (keyword_or)
  (keyword_not)
  (keyword_on)
] @keyword.operator

[
  (keyword_all)
  (keyword_add)
  (keyword_after)
  (keyword_alter)
  (keyword_analyze)
  (keyword_any)
  (keyword_as)
  (keyword_asc)
  (keyword_before)
  (keyword_begin)
  (keyword_by)
  (keyword_cascade)
  (keyword_cast)
  (keyword_check)
  (keyword_collate)
  (keyword_column)
  (keyword_columns)
  (keyword_comment)
  (keyword_commit)
  (keyword_constraint)
  (keyword_constraints)
  (keyword_concurrently)
  (keyword_current)
  (keyword_current_timestamp)
  (keyword_create)
  (keyword_cross)
  (keyword_database)
  (keyword_default)
  (keyword_deferrable)
  (keyword_deferred)
  (keyword_definer)
  (keyword_delete)
  (keyword_desc)
  (keyword_distinct)
  (keyword_do)
  (keyword_drop)
  (keyword_each)
  (keyword_explain)
  (keyword_except)
  (keyword_execute)
  (keyword_extension)
  (keyword_filter)
  (keyword_first)
  (keyword_following)
  (keyword_for)
  (keyword_foreign)
  (keyword_from)
  (keyword_full)
  (keyword_function)
  (keyword_generated)
  (keyword_group)
  (keyword_groups)
  (keyword_having)
  (keyword_if)
  (keyword_immediate)
  (keyword_index)
  (keyword_inner)
  (keyword_initially)
  (keyword_insert)
  (keyword_instead)
  (keyword_intersect)
  (keyword_into)
  (keyword_invoker)
  (keyword_join)
  (keyword_key)
  (keyword_left)
  (keyword_lateral)
  (keyword_limit)
  (keyword_lines)
  (keyword_local)
  (keyword_materialized)
  (keyword_merge)
  (keyword_natural)
  (keyword_no)
  (keyword_nowait)
  (keyword_nulls)
  (keyword_null)
  (keyword_offset)
  (keyword_only)
  (keyword_order)
  (keyword_others)
  (keyword_outer)
  (keyword_over)
  (keyword_parallel)
  (keyword_partition)
  (keyword_partitioned)
  (keyword_preceding)
  (keyword_primary)
  (keyword_procedure)
  (keyword_range)
  (keyword_recursive)
  (keyword_references)
  (keyword_rename)
  (keyword_right)
  (keyword_repeatable)
  (keyword_restrict)
  (keyword_returns)
  (keyword_rows)
  (keyword_select)
  (keyword_security)
  (keyword_session)
  (keyword_set)
  (keyword_stable)
  (keyword_table)
  (keyword_trigger)
  (keyword_truncate)
  (keyword_union)
  (keyword_unbounded)
  (keyword_unique)
  (keyword_update)
  (keyword_using)
  (keyword_values)
  (keyword_vacuum)
  (keyword_view)
  (keyword_volatile)
  (keyword_where)
  (keyword_window)
  (keyword_with)
] @keyword

[
  (keyword_bigint)
  (keyword_bigserial)
  (keyword_binary)
  (keyword_bit)
  (keyword_boolean)
  (keyword_bytea)
  (keyword_char)
  (keyword_character)
  (keyword_date)
  (keyword_datetime)
  (keyword_decimal)
  (keyword_double)
  (keyword_enum)
  (keyword_float)
  (keyword_inet)
  (keyword_int)
  (keyword_interval)
  (keyword_json)
  (keyword_jsonb)
  (keyword_mediumint)
  (keyword_money)
  (keyword_nchar)
  (keyword_numeric)
  (keyword_nvarchar)
  (keyword_precision)
  (keyword_real)
  (keyword_serial)
  (keyword_smallint)
  (keyword_text)
  (keyword_time)
  (keyword_timestamp)
  (keyword_tinyint)
  (keyword_uuid)
  (keyword_varchar)
] @type.builtin

[
  "+"
  "-"
  "*"
  "/"
  "%"
  "^"
  ":="
  "="
  "<"
  "<="
  "!="
  ">="
  ">"
  "<>"
  (op_other)
  (op_unary_other)
] @operator

[
  "("
  ")"
] @punctuation.bracket

[
  ";"
  ","
  "."
] @punctuation.delimiter
