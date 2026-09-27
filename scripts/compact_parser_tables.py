#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "zstandard>=0.25.0",
# ]
# ///
"""压缩独立 Cargo home 中所有 Tree-sitter parser 的紧凑表。"""

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

import zstandard

ROOT = Path(__file__).resolve().parent.parent
GETTER = re.compile(
    r"(?P<header>(?:TS_PUBLIC|extern) const TSLanguage \*tree_sitter_\w+\(void\) \{)\s+"
    r"static const TSLanguage language = \{(?P<fields>.*?)\n  \};\s+return &language;\s*\}",
    re.DOTALL,
)


def run(command, **kwargs):
    return subprocess.run(command, check=True, text=True, **kwargs)


def replace_array(source, declaration, replacement):
    start = source.find(declaration)
    if start < 0 or source.find(declaration, start + 1) >= 0:
        raise ValueError(f"expected one {declaration!r}")
    end = source.find("\n};", start)
    if end < 0:
        raise ValueError(f"unterminated {declaration!r}")
    return source[:start] + replacement + source[end + 3 :]


def c_array(data, width, declaration):
    if len(data) % width:
        raise ValueError(f"unaligned {declaration}")
    lines = [declaration + " = {"]
    for offset in range(0, len(data), 16 * width):
        chunk = data[offset : offset + 16 * width]
        numbers = (
            int.from_bytes(chunk[i : i + width], sys.byteorder)
            for i in range(0, len(chunk), width)
        )
        lines.append(
            "  " + ", ".join(f"0x{number:0{2 * width}x}" for number in numbers) + ","
        )
    return "\n".join([*lines, "};"])


def compile_exporter(parser, directory, include_dir):
    source = parser.read_text()
    scanner_creates = re.findall(
        r"\bvoid \*(tree_sitter_\w+_external_scanner_create)\(void\);", source
    )
    if len(scanner_creates) > 1:
        raise ValueError(f"multiple external scanners in {parser}")
    scanner_stubs = ""
    if scanner_creates:
        prefix = scanner_creates[0].removesuffix("_create")
        scanner_stubs = f"""
void *{prefix}_create(void) {{ return (void *)0; }}
void {prefix}_destroy(void *payload) {{ (void)payload; }}
bool {prefix}_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {{
  (void)payload; (void)lexer; (void)valid_symbols; return false;
}}
unsigned {prefix}_serialize(void *payload, char *buffer) {{
  (void)payload; (void)buffer; return 0;
}}
void {prefix}_deserialize(void *payload, const char *buffer, unsigned length) {{
  (void)payload; (void)buffer; (void)length;
}}
"""
    helper = directory / "export.c"
    helper.write_text(
        f'#define TREE_SITTER_HIDE_SYMBOLS\n#include "{parser.as_posix()}"\n'
        + scanner_stubs
        + (ROOT / "scripts/compact_parser_export.c").read_text()
    )
    executable = directory / ("export.exe" if os.name == "nt" else "export")
    if os.name == "nt":
        command = [
            "cl",
            "/nologo",
            "/std:c11",
            "/O1",
            "/Gy",
            "/Gw",
            f"/I{include_dir}",
            f"/Fe:{executable}",
            str(helper),
            "/link",
            "/OPT:REF",
        ]
    else:
        dead_strip = (
            "-Wl,-dead_strip" if sys.platform == "darwin" else "-Wl,--gc-sections"
        )
        command = [
            os.environ.get("KAT_HOST_CC", "cc"),
            "-O0",
            "-ffunction-sections",
            "-fdata-sections",
            dead_strip,
            "-I",
            str(include_dir),
            str(helper),
            "-o",
            str(executable),
        ]
    run(
        command,
        cwd=directory,
        stdout=None if os.name == "nt" else subprocess.DEVNULL,
    )
    return executable


def rewrite_parser(source, direct, packed, map_offset, raw_len):
    if source.count("#define LARGE_STATE_COUNT ") != 1:
        raise ValueError("unexpected LARGE_STATE_COUNT")
    source = re.sub(
        r"#define LARGE_STATE_COUNT \d+", "#define LARGE_STATE_COUNT 2", source, count=1
    )
    source = replace_array(
        source,
        "static const uint16_t ts_parse_table[",
        c_array(
            direct,
            2,
            "static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT]",
        ),
    )
    source = replace_array(
        source,
        "static const uint16_t ts_small_parse_table[] = {",
        c_array(packed, 1, "static const unsigned char ts_compact_zstd[]"),
    )
    source = replace_array(
        source, "static const uint32_t ts_small_parse_table_map[] = {", ""
    )
    matches = list(GETTER.finditer(source))
    if len(matches) != 1:
        raise ValueError(f"expected one language getter, found {len(matches)}")
    match = matches[0]
    fields = match.group("fields")
    fields = fields.replace(
        ".small_parse_table = ts_small_parse_table,", ".small_parse_table = NULL,"
    )
    fields = fields.replace(
        ".small_parse_table_map = ts_small_parse_table_map,",
        ".small_parse_table_map = NULL,",
    )
    if (
        ".small_parse_table = NULL," not in fields
        or ".small_parse_table_map = NULL," not in fields
    ):
        raise ValueError("missing compact table pointers")
    getter = f"""static TSLanguage language = {{{fields}
}};

extern bool kat_decompress_parser_table(const unsigned char *, size_t, unsigned char *, size_t);
static void ts_init_compact_table(void) {{
  unsigned char *decoded = malloc({raw_len});
  if (!decoded || !kat_decompress_parser_table(ts_compact_zstd, sizeof(ts_compact_zstd), decoded, {raw_len})) abort();
  language.small_parse_table = (const uint16_t *)decoded;
  language.small_parse_table_map = (const uint32_t *)(decoded + {map_offset});
}}
#ifdef _WIN32
static INIT_ONCE ts_compact_once = INIT_ONCE_STATIC_INIT;
static BOOL CALLBACK ts_init_compact_once(PINIT_ONCE once, PVOID parameter, PVOID *context) {{
  (void)once; (void)parameter; (void)context;
  ts_init_compact_table();
  return TRUE;
}}
#else
static pthread_once_t ts_compact_once = PTHREAD_ONCE_INIT;
#endif
{match.group("header")}
#ifdef _WIN32
  if (!InitOnceExecuteOnce(&ts_compact_once, ts_init_compact_once, NULL, NULL)) abort();
#else
  if (pthread_once(&ts_compact_once, ts_init_compact_table)) abort();
#endif
  return &language;
}}"""
    source = source[: match.start()] + getter + source[match.end() :]
    include = '#include "tree_sitter/parser.h"'
    if source.count(include) != 1:
        raise ValueError("unexpected parser header include")
    source = source.replace(
        include,
        include
        + "\n#include <stdbool.h>\n#include <stdlib.h>\n#ifdef _WIN32\n#include <windows.h>\n#else\n#include <pthread.h>\n#endif",
        1,
    )
    return source


def update_checksum(parser, crate_root):
    checksum = crate_root / ".cargo-checksum.json"
    if checksum.exists():
        record = json.loads(checksum.read_text())
        relative = parser.relative_to(crate_root).as_posix()
        if relative not in record["files"]:
            raise ValueError(f"missing checksum for {parser}")
        record["files"][relative] = hashlib.sha256(parser.read_bytes()).hexdigest()
        checksum.write_text(json.dumps(record, separators=(",", ":")))


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument(
        "--manifest-path", required=True, type=Path, help="kat 的 Cargo.toml"
    )
    cli.add_argument(
        "--cargo-home",
        required=True,
        type=Path,
        help="已执行 cargo fetch 的独立 CARGO_HOME",
    )
    args = cli.parse_args()
    cargo_home = args.cargo_home.resolve()
    default_home = (Path.home() / ".cargo").resolve()
    if (
        not cargo_home.is_dir()
        or cargo_home == default_home
        or default_home in cargo_home.parents
    ):
        cli.error("--cargo-home 必须是独立于用户默认目录的现有 Cargo home")
    environment = dict(os.environ, CARGO_HOME=str(cargo_home))
    metadata = json.loads(
        run(
            [
                "cargo",
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--manifest-path",
                str(args.manifest_path.resolve()),
            ],
            env=environment,
            capture_output=True,
        ).stdout
    )
    shared_headers = list(
        cargo_home.rglob("kat-parser-common/include/tree_sitter/parser.h")
    )
    if len(shared_headers) != 1:
        raise ValueError(
            f"expected one shared parser header, found {len(shared_headers)}"
        )
    shared_include = shared_headers[0].parent.parent
    sources = []
    for package in metadata["packages"]:
        if not package["name"].startswith("tree-sitter-"):
            continue
        crate_root = Path(package["manifest_path"]).parent.resolve()
        if not crate_root.is_relative_to(cargo_home):
            continue
        for parser in crate_root.rglob("parser.c"):
            if "#define LARGE_STATE_COUNT " in parser.read_text(errors="replace"):
                sources.append((package["name"], crate_root, parser))
    sources.sort(key=lambda entry: (entry[0], str(entry[2])))
    if not sources:
        raise ValueError("no Tree-sitter parsers found in the isolated Cargo home")
    compressor = zstandard.ZstdCompressor(level=19)
    with tempfile.TemporaryDirectory(prefix="kat-compact-") as temporary:
        for index, (name, crate_root, parser) in enumerate(sources, 1):
            work = Path(temporary) / f"{index:03d}"
            work.mkdir()
            include_dir = (
                parser.parent
                if (parser.parent / "tree_sitter/parser.h").exists()
                else shared_include
            )
            executable = compile_exporter(parser, work, include_dir)
            direct = work / "direct.bin"
            compact = work / "compact.bin"
            map_offset = int(
                run(
                    [str(executable), str(direct), str(compact)], capture_output=True
                ).stdout.strip()
            )
            raw = compact.read_bytes()
            if map_offset % 4 or map_offset >= len(raw):
                raise ValueError(f"invalid compact map for {parser}")
            parser.write_text(
                rewrite_parser(
                    parser.read_text(),
                    direct.read_bytes(),
                    compressor.compress(raw),
                    map_offset,
                    len(raw),
                )
            )
            update_checksum(parser, crate_root)
            print(
                f"{index}/{len(sources)} {name}: {len(raw)} bytes compact", flush=True
            )
    print(f"compressed {len(sources)} parsers", flush=True)


if __name__ == "__main__":
    main()
