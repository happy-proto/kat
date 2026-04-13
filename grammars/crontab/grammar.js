/**
 * @file crontab grammar for tree-sitter
 * @author sitiyou <sitiyo@outlook.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'crontab',

  extras: _ => [],

  rules: {
    // A crontab file is line-oriented; each line has one primary role.
    source_file: $ => repeat(choice(
      $.cron_job,
      $.special_job,
      $.environment_variable,
      $.comment,
      $._blank_line
    )),

    comment: $ => seq(optional($._inline_whitespace), '#', /[^\n]*/, $._newline),

    _blank_line: $ => seq(optional($._inline_whitespace), $._newline),

    environment_variable: $ => seq(
      field('name', $.identifier),
      optional($._inline_whitespace),
      '=',
      optional($._inline_whitespace),
      field('value', /[^\n]*/),
      $._newline
    ),

    special_job: $ => seq(
      field('special', $.special_time),
      $._whitespace,
      field('command', $.command),
      $._newline
    ),

    special_time: _ => choice(
      '@reboot',
      '@yearly',
      '@annually',
      '@monthly',
      '@weekly',
      '@daily',
      '@midnight',
      '@hourly'
    ),

    cron_job: $ => seq(
      field('minute', $.field),
      $._whitespace,
      field('hour', $.field),
      $._whitespace,
      field('day_of_month', $.field),
      $._whitespace,
      field('month', $.field),
      $._whitespace,
      field('day_of_week', $.field),
      $._whitespace,
      field('command', $.command),
      $._newline
    ),

    // A field can be a list (which covers ranges, steps, and singles)
    field: $ => seq(
      $._value,
      repeat(seq(',', $._value))
    ),

    _value: $ => choice(
      $.step,
      $.asterisk,
      $.range,
      $.number
    ),

    range: $ => seq($.number, '-', $.number),
    
    step: $ => seq(choice($.range, $.asterisk, $.number), '/', $.number),

    asterisk: $ => '*',
    
    number: $ => /\d+/,

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,

    command: $ => /[^\n]*/,

    _newline: $ => /\r?\n/,

    _whitespace: $ => /[ \t]+/,

    _inline_whitespace: $ => /[ \t]+/
  }
});
