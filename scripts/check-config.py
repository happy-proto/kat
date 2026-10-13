#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pyyaml>=6.0.3",
#     "rich>=15.0.0",
#     "typer>=0.27.3",
# ]
# ///

"""检查 TOML / YAML 文件语法，不修改文件。"""

import tempfile
import tomllib
import unittest
from pathlib import Path
from typing import Annotated

import typer
import yaml
from rich.text import Text
from typer.testing import CliRunner

app = typer.Typer(help=__doc__, add_completion=False, pretty_exceptions_enable=False)


def validate_file(path: Path) -> None:
    """解析单个文件；语法或读取失败时由调用方处理异常。"""
    with path.open("rb") as source:
        match path.suffix:
            case ".toml":
                tomllib.load(source)
            case ".yaml" | ".yml":
                yaml.compose(source, Loader=yaml.SafeLoader)
            case _:
                raise ValueError("expected a .toml, .yaml or .yml file")


@app.command()
def check(
    files: Annotated[
        list[Path],
        typer.Argument(
            exists=True, dir_okay=False, readable=True, help="TOML / YAML 文件"
        ),
    ],
) -> None:
    """检查所有文件，报告每个文件的错误。"""
    failed = False
    for path in files:
        try:
            validate_file(path)
        except (OSError, ValueError, yaml.YAMLError) as error:
            typer.echo(f"{path}: {error}", err=True)
            failed = True
    if failed:
        raise typer.Exit(1)


class ConfigTests(unittest.TestCase):
    """直接验证解析行为，并通过 CliRunner 验证命令行契约。"""

    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.runner = CliRunner()

    def sample(self, name: str, content: str) -> Path:
        """创建独立的临时输入文件。"""
        path = self.root / name
        path.write_text(content, encoding="utf-8")
        return path

    def test_valid_syntax(self) -> None:
        for name, content in [
            ("valid config.toml", 'name = "kat"\n'),
            ("valid.yml", "name: kat\n"),
            ("custom-tag.yaml", "name: !color red\n"),
        ]:
            with self.subTest(name=name):
                validate_file(self.sample(name, content))

    def test_invalid_syntax(self) -> None:
        for name, content, error in [
            ("broken.toml", "name = [\n", tomllib.TOMLDecodeError),
            ("broken.yml", "name: [\n", yaml.YAMLError),
            ("multiple.yaml", "---\nname: first\n---\nname: second\n", yaml.YAMLError),
            ("unsupported.txt", "name: kat\n", ValueError),
        ]:
            with self.subTest(name=name), self.assertRaises(error):
                validate_file(self.sample(name, content))

    def test_cli_reports_all_errors(self) -> None:
        paths = [
            self.sample("broken.toml", "name = [\n"),
            self.sample("broken.yml", "[\n"),
        ]
        result = self.runner.invoke(app, ["check", *map(str, paths)])
        self.assertEqual(result.exit_code, 1)
        for path in paths:
            self.assertIn(f"{path}:", result.stderr)

    def test_cli_arguments(self) -> None:
        path = str(self.sample("valid config.toml", 'name = "kat"\n'))
        for arguments, status in [
            (["check", path], 0),
            (["check"], 2),
            (["check", str(self.root / "missing.toml")], 2),
            (["check", str(self.root)], 2),
            (["self-test", path], 2),
        ]:
            with self.subTest(arguments=arguments):
                self.assertEqual(self.runner.invoke(app, arguments).exit_code, status)

    def test_help(self) -> None:
        result = self.runner.invoke(app, ["--help"])
        self.assertEqual(result.exit_code, 0)
        output = Text.from_ansi(result.output).plain
        self.assertIn("check", output)
        self.assertIn("self-test", output)


@app.command()
def self_test() -> None:
    """运行文件解析和命令行回归测试。"""
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ConfigTests)
    result = unittest.TextTestRunner().run(suite)
    if not result.wasSuccessful():
        raise typer.Exit(1)


if __name__ == "__main__":
    app()
