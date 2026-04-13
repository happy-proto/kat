module.exports = grammar({
  name: "cabal",

  extras: ($) => [$.silly, /\s|\n/],

  externals: ($) => [$.silly, $.indent, $.dedent, $._indented],

  conflicts: ($) => [[$.properties], [$.sections]],

  rules: {
    cabal: ($) =>
      seq(
        optional($.cabal_version),
        $.properties,
        $.sections,
        repeat($.comment),
      ),

    cabal_version: ($) => seq("cabal-version", ":", $.spec_version),

    spec_version: ($) => /\d+\.\d+(\.\d+)?\s*\n/,

    properties: ($) => repeat1(seq(repeat($.comment), $.field)),

    sections: ($) =>
      repeat1(
        choice(
          $.benchmark,
          $.common,
          $.executable,
          $.flag,
          $.library,
          $.source_repository,
          $.test_suite,
        ),
      ),

    benchmark: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("benchmark", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_block),
      ),

    common: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("common", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_block),
      ),

    executable: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("executable", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_or_conditional_block),
      ),

    flag: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("flag", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_block),
      ),

    library: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("library", $.section_type)),
        optional(field("name", $.section_name)),
        field("properties", $.property_or_conditional_block),
      ),

    source_repository: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("source-repository", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_block),
      ),

    test_suite: ($) =>
      seq(
        repeat($.comment),
        field("type", alias("test-suite", $.section_type)),
        field("name", $.section_name),
        field("properties", $.property_block),
      ),

    section_name: ($) => /\d*[a-zA-Z]\w*(-\d*[a-zA-Z]\w*)*/,

    comment: ($) => token(seq("--", /.*/)),

    property_block: ($) =>
      seq(
        $.indent,
        repeat($.comment),
        repeat1(seq($.field, repeat($.comment))),
        $.dedent,
      ),

    field: ($) =>
      seq(
        $.field_name,
        ":",
        $.field_value,
        optional(
          seq(
            $.indent,
            $.field_value,
            repeat(seq($._indented, $.field_value)),
            $.dedent,
          ),
        ),
      ),

    field_name: ($) => /\w(\w|-)+/,

    field_value: ($) => /.+/,

    property_or_conditional_block: ($) =>
      seq(
        $.indent,
        repeat($.comment),
        repeat1(seq(choice($.field, $.conditional), repeat($.comment))),
        $.dedent,
      ),

    conditional: ($) =>
      seq(
        $.condition_if,
        repeat($.condition_elseif),
        optional($.condition_else),
      ),

    condition_if: ($) =>
      seq("if", $.condition, $.property_or_conditional_block),
    condition_elseif: ($) =>
      seq("elseif", $.condition, $.property_or_conditional_block),
    condition_else: ($) => seq("else", $.property_or_conditional_block),
    condition: ($) => /.*/,
  },
});
