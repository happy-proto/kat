/**
 * @file Sass (indented syntax) grammar for tree-sitter
 * @author Raunak
 * @license MIT
 *
 * Sass indented syntax uses indentation instead of braces and
 * newlines instead of semicolons, distinguishing it from SCSS.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-nocheck

module.exports = grammar({
  name: 'sass',

  externals: ($) => [
    $._indent,
    $._dedent,
    $._newline,
    $._string_content,
    $._error_sentinel,
  ],

  extras: ($) => [
    /[ \t]/,
    $.comment,
    $.single_line_comment,
  ],

  word: ($) => $._identifier_token,

  conflicts: ($) => [
    [$._selector, $._identifier_with_interpolation],
    [$._query, $._identifier_with_interpolation],
    [$.include_statement, $._identifier_with_interpolation],
    [$.container_statement, $._identifier_with_interpolation],
    [$._range_operator, $._binary_operator],
    [$.tag_name, $._value],
  ],

  inline: ($) => [$._top_level_item, $._block_item],

  rules: {
    stylesheet: ($) => repeat($._top_level_item),

    _top_level_item: ($) =>
      choice(
        $.declaration,
        $.rule_set,
        $.import_statement,
        $.use_statement,
        $.forward_statement,
        $.media_statement,
        $.charset_statement,
        $.namespace_statement,
        $.keyframes_statement,
        $.supports_statement,
        $.scope_statement,
        $.layer_statement,
        $.container_statement,
        $.font_face_statement,
        $.mixin_statement,
        $.include_statement,
        $.if_statement,
        $.each_statement,
        $.for_statement,
        $.while_statement,
        $.function_statement,
        $.error_statement,
        $.warn_statement,
        $.debug_statement,
        $.at_root_statement,
        $.at_rule,
        $.placeholder,
        $._newline,
      ),

    // === Statements ===

    import_statement: ($) =>
      prec.right(
        seq(
          '@import',
          sep1(',', $._value),
          optional(sep1(',', $._query)),
          $._newline,
        ),
      ),

    use_statement: ($) =>
      seq(
        '@use',
        $._value,
        optional($.as_clause),
        optional($.with_clause),
        $._newline,
      ),

    forward_statement: ($) =>
      seq(
        '@forward',
        $._value,
        optional($.as_clause),
        optional($.visibility_clause),
        optional($.with_clause),
        $._newline,
      ),

    media_statement: ($) =>
      seq('@media', sep1(',', $._query), $._newline, optional($.block)),

    charset_statement: ($) => seq('@charset', $._value, $._newline),

    namespace_statement: ($) =>
      seq(
        '@namespace',
        optional(alias($._identifier, $.namespace_name)),
        choice($.string_value, $.call_expression),
        $._newline,
      ),

    font_face_statement: ($) =>
      seq('@font-face', $._newline, optional($.block)),

    keyframes_statement: ($) =>
      prec.right(
        1,
        seq(
          alias(/@(-[a-z]+-)?keyframes/, $.at_keyword),
          field('name', alias($._plain_value_with_interpolation, $.keyframes_name)),
          $._newline,
          optional(alias($.keyframe_block_list, $.block)),
        ),
      ),

    keyframe_block_list: ($) =>
      seq($._indent, repeat($.keyframe_block), $._dedent),

    keyframe_block: ($) =>
      seq(
        choice($.from, $.to, $.integer_value, $.float_value),
        $._newline,
        optional($.block),
      ),

    from: (_) => 'from',
    to: (_) => 'to',

    supports_statement: ($) =>
      seq('@supports', $._query, $._newline, optional($.block)),

    scope_statement: ($) =>
      seq(
        '@scope',
        optional(
          seq(
            '(',
            $._selector,
            ')',
            optional(seq('to', '(', $._selector, ')')),
          ),
        ),
        $._newline,
        optional($.block),
      ),

    layer_statement: ($) =>
      prec.right(
        seq(
          '@layer',
          choice(
            seq(sep1(',', alias($._identifier, $.layer_name)), $._newline),
            seq(
              optional(alias($._identifier, $.layer_name)),
              $._newline,
              optional($.block),
            ),
          ),
        ),
      ),

    container_statement: ($) =>
      seq(
        '@container',
        optional(alias($._identifier, $.container_name)),
        $._query,
        $._newline,
        optional($.block),
      ),

    at_rule: ($) =>
      prec.right(
        seq(
          $.at_keyword,
          optional(sep1(',', $._query)),
          $._newline,
          optional($.block),
        ),
      ),

    // === @use and @forward clauses ===

    as_clause: ($) => seq('as', $.use_alias),

    use_alias: (_) =>
      choice('*', /[a-zA-Z_-][a-zA-Z0-9_-]*/, /[a-zA-Z_-][a-zA-Z0-9_-]*-\*/),

    with_clause: ($) => seq('with', $.with_parameters),

    visibility_clause: ($) =>
      seq(choice('hide', 'show'), $.visibility_parameters),

    visibility_parameters: ($) =>
      sep1(
        ',',
        choice(
          alias(/[a-zA-Z_-][a-zA-Z0-9_-]*/, $.identifier),
          alias($.variable_identifier, $.variable_value),
        ),
      ),

    // === Parameters ===

    parameters: ($) =>
      seq(
        '(',
        choice(
          sep(',', $.parameter),
          seq(sep1(',', $.parameter), ',', $.rest_parameter),
          $.rest_parameter,
        ),
        ')',
      ),

    rest_parameter: ($) =>
      seq(alias($.variable_identifier, $.variable_name), $._spread),

    _spread: (_) => alias(token.immediate('...'), '...'),

    parameter: ($) =>
      seq(
        alias($.variable_identifier, $.variable_name),
        optional(seq(':', alias($._value, $.default_value))),
      ),

    with_parameters: ($) =>
      seq('(', sep1(',', $.with_parameter), optional(','), ')'),

    with_parameter: ($) =>
      seq(
        alias($.variable_identifier, $.variable_name),
        ':',
        $._value,
        optional($.default),
      ),

    // === Mixin and Include ===

    mixin_statement: ($) =>
      prec.right(
        seq(
          choice('@mixin', alias('=', $.shorthand_mixin)),
          alias($._identifier, $.name),
          optional($.parameters),
          $._newline,
          optional($.block),
        ),
      ),

    include_statement: ($) =>
      prec.right(
        seq(
          choice('@include', alias('+', $.shorthand_include)),
          optional(
            seq(
              field('module', alias($._identifier, $.module)),
              token.immediate('.'),
            ),
          ),
          alias($._identifier, $.mixin_name),
          optional(alias($.include_arguments, $.arguments)),
          optional($.using_clause),
          $._newline,
          optional($.block),
        ),
      ),

    include_arguments: ($) =>
      seq(
        token.immediate('('),
        sep(',', alias($.include_argument, $.argument)),
        ')',
      ),

    include_argument: ($) =>
      seq(
        optional(seq(alias($.variable_identifier, $.argument_name), ':')),
        alias($._value, $.argument_value),
      ),

    using_clause: ($) => seq('using', $.parameters),

    // === Placeholder ===

    placeholder_declaration_selector: ($) =>
      seq('%', alias($._identifier_with_interpolation, $.placeholder_name)),

    placeholder: ($) =>
      seq(
        alias($.placeholder_declaration_selector, $.placeholder_selector),
        $._newline,
        optional($.block),
      ),

    // === Extend ===

    extend_statement: ($) =>
      seq(
        '@extend',
        choice($._value, $.class_selector, $.placeholder_selector, $.id_selector),
        optional($.optional_flag),
        $._newline,
      ),

    optional_flag: (_) => '!optional',

    // === Control Flow ===

    if_statement: ($) =>
      seq($.if_clause, repeat($.else_if_clause), optional($.else_clause)),

    if_clause: ($) =>
      seq('@if', alias($._value, $.condition), $._newline, optional($.block)),

    else_if_clause: ($) =>
      seq(
        '@else',
        'if',
        alias($._value, $.condition),
        $._newline,
        optional($.block),
      ),

    else_clause: ($) => seq('@else', $._newline, optional($.block)),

    each_statement: ($) =>
      seq(
        '@each',
        sep1(',', alias($.variable_identifier, $.variable)),
        'in',
        $._value,
        $._newline,
        optional($.block),
      ),

    for_statement: ($) =>
      seq(
        '@for',
        alias($.variable_identifier, $.variable_name),
        'from',
        alias($._value, $.from_value),
        choice('through', 'to'),
        alias($._value, $.to_value),
        $._newline,
        optional($.block),
      ),

    while_statement: ($) =>
      seq('@while', alias($._value, $.condition), $._newline, optional($.block)),

    // === Functions ===

    function_statement: ($) =>
      seq(
        '@function',
        alias($._identifier, $.name),
        optional($.parameters),
        $._newline,
        optional($.block),
      ),

    return_statement: ($) => seq('@return', $._value, $._newline),

    // === At-root ===

    at_root_statement: ($) =>
      seq(
        '@at-root',
        optional(choice($.at_root_query, $.selectors)),
        $._newline,
        optional($.block),
      ),

    at_root_query: ($) => seq('(', choice('with', 'without'), ':', $._value, ')'),

    // === Logging ===

    error_statement: ($) => seq('@error', $._value, $._newline),
    warn_statement: ($) => seq('@warn', $._value, $._newline),
    debug_statement: ($) => seq('@debug', $._value, $._newline),

    // === Content ===

    content_statement: ($) =>
      seq('@content', optional($.content_arguments), $._newline),

    content_arguments: ($) => seq('(', sep(',', $._value), ')'),

    // === Rule Sets ===

    rule_set: ($) => prec.right(seq($.selectors, $._newline, optional($.block))),

    selectors: ($) =>
      sep1(',', choice($._selector, $._block_direct_selector)),

    block: ($) =>
      seq($._indent, repeat($._block_item), $._dedent),

    _block_item: ($) =>
      choice(
        $.declaration,
        $.rule_set,
        $.import_statement,
        $.use_statement,
        $.forward_statement,
        $.media_statement,
        $.charset_statement,
        $.namespace_statement,
        $.keyframes_statement,
        $.supports_statement,
        $.scope_statement,
        $.layer_statement,
        $.container_statement,
        $.font_face_statement,
        $.mixin_statement,
        $.include_statement,
        $.extend_statement,
        $.if_statement,
        $.each_statement,
        $.for_statement,
        $.while_statement,
        $.function_statement,
        $.return_statement,
        $.at_root_statement,
        $.error_statement,
        $.warn_statement,
        $.debug_statement,
        $.content_statement,
        $.at_rule,
        $._newline,
      ),

    // === Selector Combinators ===

    _block_direct_selector: ($) =>
      choice(
        alias($._block_direct_child_selector, $.child_selector),
        alias($._block_direct_sibling_selector, $.sibling_selector),
        alias(
          $._block_direct_adjacent_sibling_selector,
          $.adjacent_sibling_selector,
        ),
      ),

    _block_direct_child_selector: ($) => seq('>', field('right', $._selector)),

    _block_direct_sibling_selector: ($) => seq('~', field('right', $._selector)),

    _block_direct_adjacent_sibling_selector: ($) =>
      seq('+', field('right', $._selector)),

    // === Selectors ===

    _selector: ($) =>
      prec.left(
        choice(
          $.universal_selector,
          $.tag_name,
          $.class_selector,
          $.nesting_selector,
          $.pseudo_class_selector,
          $.pseudo_element_selector,
          $.id_selector,
          $.attribute_selector,
          $.string_value,
          $.child_selector,
          $.descendant_selector,
          $.sibling_selector,
          $.adjacent_sibling_selector,
          $.namespace_selector,
          $.interpolation,
          $.placeholder_selector,
        ),
      ),

    nesting_selector: ($) =>
      prec.right(
        seq(
          '&',
          optional(alias($._identifier_suffix, $.suffix)),
        ),
      ),

    _identifier_suffix: (_) => token.immediate(/[-_a-zA-Z0-9]+/),

    universal_selector: (_) => '*',

    tag_name: ($) =>
      prec.left(-1, $._identifier_with_interpolation),

    placeholder_selector: ($) =>
      prec(
        1,
        seq('%', alias($._identifier_with_interpolation, $.placeholder_name)),
      ),

    class_selector: ($) =>
      prec(
        1,
        seq(
          optional($._selector),
          choice('.', seq($.nesting_selector)),
          alias($._identifier_with_interpolation, $.class_name),
        ),
      ),

    pseudo_class_selector: ($) =>
      prec(
        1,
        seq(
          optional($._selector),
          token(':'),
          choice(
            $._nth_child_pseudo_class_selector,
            seq(
              alias(
                choice('has', 'not', 'is', 'where', 'host', 'host-context'),
                $.class_name,
              ),
              alias($.pseudo_class_with_selector_arguments, $.arguments),
            ),
            seq(
              alias($._identifier_with_interpolation, $.class_name),
              optional(alias($.pseudo_class_arguments, $.arguments)),
            ),
            alias('host', $.class_name),
          ),
        ),
      ),

    _nth_child_pseudo_class_selector: ($) =>
      prec(
        2,
        seq(
          alias(
            choice('nth-child', 'nth-last-child', 'nth-of-type', 'nth-last-of-type'),
            $.class_name,
          ),
          alias($.pseudo_class_nth_child_arguments, $.arguments),
        ),
      ),

    pseudo_element_selector: ($) =>
      seq(
        optional($._selector),
        '::',
        alias($._identifier_with_interpolation, $.element_name),
        optional(alias($.pseudo_element_arguments, $.arguments)),
      ),

    pseudo_element_arguments: ($) =>
      seq(token.immediate('('), sep(',', choice($._selector, repeat1($._value))), ')'),

    id_selector: ($) =>
      prec(
        1,
        seq(
          optional($._selector),
          '#',
          alias($._identifier_with_interpolation, $.id_name),
        ),
      ),

    attribute_selector: ($) =>
      seq(
        optional($._selector),
        '[',
        alias(
          choice($._identifier_with_interpolation, $.namespace_selector),
          $.attribute_name,
        ),
        optional(
          seq(choice('=', '~=', '^=', '|=', '*=', '$='), $._value, optional(/[iIsS]/)),
        ),
        ']',
      ),

    child_selector: ($) =>
      prec.left(seq(field('left', $._selector), '>', field('right', $._selector))),

    descendant_selector: ($) =>
      prec.left(
        seq(field('left', $._selector), /[ \t]+/, field('right', $._selector)),
      ),

    sibling_selector: ($) =>
      prec.left(seq(field('left', $._selector), '~', field('right', $._selector))),

    adjacent_sibling_selector: ($) =>
      prec.left(seq(field('left', $._selector), '+', field('right', $._selector))),

    namespace_selector: ($) =>
      prec.left(seq($._selector, '|', $._selector)),

    pseudo_class_arguments: ($) =>
      seq(
        token.immediate('('),
        sep(',', choice(prec.dynamic(1, $._selector), repeat1($._value))),
        ')',
      ),

    pseudo_class_with_selector_arguments: ($) =>
      seq(
        token.immediate('('),
        sep(',', $._selector),
        ')',
      ),

    pseudo_class_nth_child_arguments: ($) =>
      seq(
        token.immediate('('),
        choice(
          alias('even', $.plain_value),
          alias('odd', $.plain_value),
          alias(token(prec(3, /-?(\d)*n(\s*[+-]\s*\d+)?/)), $.plain_value),
          $.integer_value,
        ),
        optional(seq('of', $._selector)),
        ')',
      ),

    // === Declarations ===

    declaration: ($) =>
      prec.right(
        1,
        choice(
          seq(
            alias($.variable_identifier, $.variable_name),
            ':',
            $._value,
            repeat(seq(optional(','), $._value)),
            optional(choice($.default, $.global)),
            $._newline,
          ),
          seq(
            alias($.property_name_token, $.property_name),
            ':',
            $._value,
            repeat(seq(optional(','), $._value)),
            optional($.important),
            $._newline,
            optional($.block),
          ),
        ),
      ),

    property_name_token: ($) =>
      $._identifier_with_interpolation,

    important: (_) => '!important',
    default: (_) => '!default',
    global: (_) => '!global',

    // === Media Queries ===

    _query: ($) =>
      choice(
        alias($._identifier_with_interpolation, $.keyword_query),
        $.feature_query,
        $.range_query,
        $.binary_query,
        $.unary_query,
        $.selector_query,
        $.parenthesized_query,
        $.interpolation,
      ),

    feature_query: ($) =>
      prec(-1, seq('(', alias($._identifier, $.feature_name), ':', repeat1($._value), ')')),

    parenthesized_query: ($) => seq('(', $._query, ')'),

    binary_query: ($) =>
      prec.left(seq($._query, choice('and', 'or'), $._query)),

    unary_query: ($) => prec(1, seq(choice('not', 'only'), $._query)),

    range_query: ($) =>
      prec(
        2,
        seq(
          '(',
          choice(
            seq(alias($._identifier, $.feature_name), $._range_operator, $._value),
            seq($._value, $._range_operator, alias($._identifier, $.feature_name)),
            seq(
              $._value,
              $._range_operator,
              alias($._identifier, $.feature_name),
              $._range_operator,
              $._value,
            ),
          ),
          ')',
        ),
      ),

    _range_operator: (_) => prec(3, choice('<', '>', '<=', '>=', '=')),

    selector_query: ($) => seq('selector', '(', $._selector, ')'),

    // === Property Values ===

    _value: ($) =>
      prec(
        -1,
        choice(
          alias($._identifier_with_interpolation, $.plain_value),
          $.variable_module,
          alias($.variable_identifier, $.variable_value),
          $.css_variable,
          $.boolean_value,
          $.null_value,
          $.color_value,
          $.unicode_range,
          $.integer_value,
          $.float_value,
          $.string_value,
          $.list_value,
          $.map_value,
          $.binary_expression,
          $.unary_expression,
          $.parenthesized_value,
          $.call_expression,
          alias($.nesting_selector, $.nesting_value),
        ),
      ),

    boolean_value: (_) => choice('true', 'false'),
    null_value: (_) => 'null',

    css_variable: ($) =>
      seq(
        alias(token(seq('--', /[a-zA-Z_-][a-zA-Z0-9_-]*/)), $.custom_property_name),
      ),

    parenthesized_value: ($) => seq('(', $._value, ')'),

    map_value: ($) =>
      prec(1, seq('(', sep1(',', $.map_pair), optional(','), ')')),

    map_pair: ($) =>
      prec(
        2,
        seq(
          field('key', $._value),
          ':',
          field('value', $._value),
          optional($.default),
        ),
      ),

    list_value: ($) =>
      prec.right(-1, seq('[', sep(',', $._value), ']')),

    unicode_range: (_) =>
      token(
        seq(
          /[Uu]\+/,
          choice(
            /[0-9a-fA-F]{1,6}/,
            seq(/[0-9a-fA-F]{1,5}/, /\?{1,5}/),
            seq(/[0-9a-fA-F]{1,6}/, '-', /[0-9a-fA-F]{1,6}/),
          ),
        ),
      ),

    color_value: (_) => seq('#', token.immediate(/[0-9a-fA-F]{3,8}/)),

    color_function: ($) =>
      seq(
        field(
          'name',
          alias(
            token(prec(1, /rgba?|hsla?|hwb|lab|lch|oklch|oklab|color(-mix)?/)),
            $.function_name,
          ),
        ),
        field('arguments', $.arguments),
      ),

    gradient_function: ($) =>
      seq(
        field(
          'name',
          alias(
            token(prec(1, /(repeating-)?(linear|radial|conic)-gradient/)),
            $.function_name,
          ),
        ),
        field('arguments', $.arguments),
      ),

    math_function: ($) =>
      seq(
        field(
          'name',
          alias(
            token(prec(1, /calc|min|max|clamp|abs|sign|round|mod|rem|sin|cos|tan|asin|acos|atan2?|pow|sqrt|hypot|log|exp/)),
            $.function_name,
          ),
        ),
        field('arguments', $.arguments),
      ),

    var_function: ($) =>
      seq(
        field('name', alias('var', $.function_name)),
        token.immediate('('),
        alias(token(seq('--', /[a-zA-Z_-][a-zA-Z0-9_-]*/)), $.custom_property_name),
        optional(seq(',', $._value)),
        ')',
      ),

    string_value: ($) => choice($._single_quoted_string, $._double_quoted_string),

    _single_quoted_string: ($) =>
      seq("'", repeat(choice($._string_content, $.interpolation, $.escape_sequence)), "'"),

    _double_quoted_string: ($) =>
      seq('"', repeat(choice($._string_content, $.interpolation, $.escape_sequence)), '"'),

    escape_sequence: (_) =>
      token.immediate(
        seq(
          '\\',
          choice(
            /[0-9a-fA-F]{1,6}\s?/,
            /[^0-9a-fA-F\n\r]/,
          ),
        ),
      ),

    integer_value: ($) =>
      prec(
        2,
        seq(
          token(prec(2, seq(optional(choice('+', '-')), /\d+/))),
          optional($.unit),
        ),
      ),

    float_value: ($) =>
      prec(
        2,
        seq(
          token(
            prec(
              2,
              seq(
                optional(choice('+', '-')),
                /\d*/,
                choice(
                  seq('.', /\d+/),
                  seq(/[eE]/, optional('-'), /\d+/),
                  seq('.', /\d+/, /[eE]/, optional('-'), /\d+/),
                ),
              ),
            ),
          ),
          optional($.unit),
        ),
      ),

    unit: (_) => token.immediate(prec(3, /[a-zA-Z%]+/)),

    // === Expressions ===

    call_expression: ($) =>
      choice(
        $.color_function,
        $.gradient_function,
        $.math_function,
        $.var_function,
        seq(
          field('name', alias('url', $.function_name)),
          field('arguments', alias($.arguments_for_url, $.arguments)),
        ),
        seq(
          optional(
            seq(
              field('module', alias($._identifier, $.module)),
              token.immediate('.'),
            ),
          ),
          field('name', alias($._identifier, $.function_name)),
          field('arguments', $.arguments),
        ),
      ),

    _binary_operator: (_) =>
      prec(
        3,
        choice('+', '-', '*', '/', '%', '==', '<', '>', '!=', '<=', '>=', 'and', 'or'),
      ),

    _unary_operator: (_) => prec(2, choice('not', '+', '-', '/')),

    binary_expression: ($) =>
      prec.left(
        2,
        seq(
          field('left', $._value),
          field('operator', $._binary_operator),
          field('right', $._value),
        ),
      ),

    unary_expression: ($) =>
      prec.left(2, seq(field('operator', $._unary_operator), $._value)),

    arguments: ($) =>
      seq(
        token.immediate('('),
        choice(
          sep(choice(',', ';'), repeat($._value)),
          seq(sep1(choice(',', ';'), repeat($._value)), ',', $.rest_argument),
          $.rest_argument,
        ),
        ')',
      ),

    rest_argument: ($) =>
      seq(alias($.variable_identifier, $.variable_value), $._spread),

    arguments_for_url: ($) =>
      seq(
        token.immediate('('),
        optional(
          choice(
            $.string_value,
            alias($.unquoted_url_value, $.plain_value),
            $.variable_identifier,
            $.interpolation,
          ),
        ),
        ')',
      ),

    unquoted_url_value: (_) =>
      token(
        seq(
          repeat(choice(/[-_]/, /\\[^\s,;!{}()\[\]]/)),
          /[a-zA-Z]/,
          repeat(choice(/[^\/\s,;!{}()\[\]]/, /\\[^\s,;!{}()\[\]]/)),
        ),
      ),

    variable_module: ($) =>
      seq(
        field('module', alias($._identifier, $.module)),
        token.immediate('.'),
        field('value', alias($.variable_identifier, $.variable_value)),
      ),

    // === Identifiers ===

    _identifier: ($) => $._identifier_token,

    _identifier_token: (_) => /((--|-?[a-zA-Z_]))([a-zA-Z0-9\-_])*/,

    variable_identifier: (_) => /\$[a-zA-Z_-][a-zA-Z0-9_-]*/,

    at_keyword: (_) => /@[a-zA-Z_-][a-zA-Z0-9_-]*/,

    // === Comments ===

    comment: (_) => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

    single_line_comment: (_) => token(seq('//', /.*/)),

    sassdoc_comment: (_) => token(seq('///', /.*/)),

    // === Interpolation ===

    interpolation: ($) => seq('#{', choice($._value, $.binary_expression), '}'),

    _identifier_with_interpolation: ($) =>
      prec.left(1, repeat1(choice($._identifier, $.interpolation))),

    _plain_value_with_interpolation: ($) =>
      repeat1(choice($._plain_value, $.interpolation)),

    _plain_value: (_) =>
      token(seq(repeat(/[-_]/), /[a-zA-Z]/, repeat(/[a-zA-Z0-9_-]/))),
  },
});

function sep(separator, rule) {
  return optional(sep1(separator, rule));
}

function sep1(separator, rule) {
  return seq(rule, repeat(seq(separator, rule)));
}
