#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pyyaml>=6.0.3",
# ]
# ///

"""检查 TOML / YAML 文件语法，不修改文件。"""

import argparse
import sys
import tomllib
from pathlib import Path

import yaml


def main() -> int:
    """校验所有输入文件，逐个报告错误。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="+", type=Path, help="要校验的 TOML / YAML 文件")
    failed = False
    for path in parser.parse_args().files:
        try:
            with path.open("rb") as source:
                if path.suffix == ".toml":
                    tomllib.load(source)
                elif path.suffix in {".yaml", ".yml"}:
                    yaml.compose(source, Loader=yaml.SafeLoader)
                else:
                    raise ValueError("expected a .toml, .yaml or .yml file")
        except (OSError, ValueError, yaml.YAMLError) as error:
            print(f"{path}: {error}", file=sys.stderr)
            failed = True
    return int(failed)


if __name__ == "__main__":
    sys.exit(main())
