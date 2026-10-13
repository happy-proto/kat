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
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

import yaml


def check_files(files: list[Path]) -> int:
    """校验所有输入文件，逐个报告错误。"""
    failed = False
    for path in files:
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


def self_test() -> int:
    """通过真实 CLI 验证语法校验和参数边界。"""
    samples = {
        "valid config.toml": 'name = "kat"\n',
        "custom-tag.yaml": "name: !color red\n",
        "broken.toml": "name = [\n",
        "broken.yml": "name: [\n",
        "multiple.yaml": "---\nname: first\n---\nname: second\n",
    }
    cases = [
        (["valid config.toml", "custom-tag.yaml"], 0, []),
        (["broken.toml", "broken.yml"], 1, ["broken.toml:", "broken.yml:"]),
        (["multiple.yaml"], 1, ["multiple.yaml:"]),
        (["missing.toml"], 1, ["missing.toml:"]),
        (["--help"], 0, ["--self-test"]),
        ([], 2, ["error:"]),
        (["--self-test", "valid config.toml"], 2, ["error:"]),
    ]
    with tempfile.TemporaryDirectory() as scratch:
        for name, content in samples.items():
            Path(scratch, name).write_text(content, encoding="utf-8")
        for arguments, expected_status, expected_messages in cases:
            result = subprocess.run(
                [sys.executable, str(Path(__file__).resolve()), *arguments],
                cwd=scratch,
                capture_output=True,
                text=True,
                check=False,
            )
            output = result.stdout + result.stderr
            if result.returncode != expected_status or any(
                message not in output for message in expected_messages
            ):
                print(f"Self-test failed: {arguments!r}\n{output}", file=sys.stderr)
                return 1
    print("Config syntax checks passed")
    return 0


def main() -> int:
    """执行文件校验或内建回归测试。"""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="*", type=Path, help="要校验的 TOML / YAML 文件")
    parser.add_argument("--self-test", action="store_true", help="运行内建回归测试")
    arguments = parser.parse_args()
    if arguments.self_test:
        if arguments.files:
            parser.error("--self-test cannot be combined with files")
        return self_test()
    if not arguments.files:
        parser.error("provide files or --self-test")
    return check_files(arguments.files)


if __name__ == "__main__":
    sys.exit(main())
