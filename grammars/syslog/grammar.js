module.exports = grammar({
  name: "syslog",

  extras: (_) => [],

  rules: {
    source_file: ($) => repeat(choice($.entry, $.comment, $._blank_line)),

    entry: ($) =>
      seq(
        field("month", $.month),
        $._gap,
        field("day", $.number),
        $._gap,
        field("time", $.time),
        $._gap,
        field("host", $.hostname),
        $._gap,
        field("process", $.process),
        ":",
        optional(seq($._gap, field("message", $.message))),
        $._newline,
      ),

    process: ($) =>
      seq(
        field("name", $.process_name),
        optional(seq("[", field("pid", $.number), "]")),
      ),

    comment: ($) => seq("#", /[^\n]*/, $._newline),

    month: (_) =>
      choice(
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ),
    number: (_) => /[0-9]+/,
    time: (_) => /[0-9]{2}:[0-9]{2}:[0-9]{2}/,
    hostname: (_) => /[A-Za-z0-9._-]+/,
    process_name: (_) => /[A-Za-z0-9_.@/-]+/,
    message: (_) => /[^\n]*/,

    _gap: (_) => /[ \t]+/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
