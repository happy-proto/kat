#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "rich>=15.0.0",
#     "typer>=0.27.3",
# ]
# ///

"""从发布归档生成可执行文件体积表，不解压或执行二进制。"""

import io
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from typing import Annotated

import typer
from rich.text import Text
from typer.testing import CliRunner

app = typer.Typer(help=__doc__, add_completion=False, pretty_exceptions_enable=False)


def size_table(asset_dir: Path) -> str:
    """按平台和架构列出发布包内 kat 的未压缩大小。"""
    packages = sorted([*asset_dir.glob("kat-*.tar.xz"), *asset_dir.glob("kat-*.zip")])
    if not packages:
        raise ValueError("没有找到 kat 发布包")
    platforms = {
        "unknown-linux-gnu": "Linux（GNU）",
        "apple-darwin": "macOS",
        "pc-windows-msvc": "Windows",
    }
    rows = []
    for package in packages:
        windows = package.suffix == ".zip"
        stem = package.name.removesuffix(".zip" if windows else ".tar.xz")
        target = stem.removeprefix("kat-")
        architecture, _, platform = target.partition("-")
        if platform not in platforms:
            raise ValueError(f"无法识别发布平台：{target}")
        member_name = f"{stem}/kat{'.exe' if windows else ''}"
        if windows:
            with zipfile.ZipFile(package) as archive:
                member = archive.getinfo(member_name)
                if member.is_dir():
                    raise ValueError(f"{package.name} 中的 kat.exe 不是普通文件")
                size = member.file_size
        else:
            with tarfile.open(package, "r:xz") as archive:
                member = archive.getmember(member_name)
                if not member.isfile():
                    raise ValueError(f"{package.name} 中的 kat 不是普通文件")
                size = member.size
        architecture = "ARM64" if architecture == "aarch64" else architecture
        rows.append(
            f"| {platforms[platform]} | {architecture} | {size / (1024**2):.2f} MiB |"
        )
    return "\n".join(
        [
            "<details>",
            "<summary>二进制体积</summary>",
            "",
            "按安装到目标设备后的可执行文件大小统计。",
            "",
            "| 平台 | 架构 | 体积 |",
            "| --- | --- | ---: |",
            *rows,
            "",
            "</details>",
        ]
    )


@app.command()
def report(
    asset_dir: Annotated[
        Path,
        typer.Argument(
            metavar="ASSET_DIR",
            exists=True,
            file_okay=False,
            readable=True,
            resolve_path=True,
            help="包含 kat-*.tar.xz 和 kat-*.zip 发布包的目录。",
        ),
    ],
) -> None:
    """输出折叠 Markdown 表格，体积以 MiB 保留两位小数。"""
    try:
        table = size_table(asset_dir)
    except (
        OSError,
        ValueError,
        KeyError,
        tarfile.TarError,
        zipfile.BadZipFile,
    ) as error:
        typer.echo(f"无法统计发布二进制体积：{error}", err=True)
        raise typer.Exit(1) from error
    typer.echo(table)


class ReleaseSizeTests(unittest.TestCase):
    """验证真实归档的统计口径与 CLI 错误边界。"""

    def test_all_release_platforms_use_uncompressed_binary_sizes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for target, mib in [
                ("x86_64-unknown-linux-gnu", 2),
                ("aarch64-apple-darwin", 3),
                ("x86_64-pc-windows-msvc", 4),
                ("aarch64-pc-windows-msvc", 5),
            ]:
                stem = f"kat-{target}"
                data = bytes(mib * 1024**2)
                if "windows" in target:
                    with zipfile.ZipFile(
                        root / f"{stem}.zip", "w", compression=zipfile.ZIP_DEFLATED
                    ) as archive:
                        archive.writestr(f"{stem}/kat.exe", data)
                else:
                    with tarfile.open(root / f"{stem}.tar.xz", "w:xz") as archive:
                        info = tarfile.TarInfo(f"{stem}/kat")
                        info.size = len(data)
                        archive.addfile(info, io.BytesIO(data))
            result = CliRunner().invoke(app, ["report", str(root)])
            self.assertEqual(result.exit_code, 0, result.output)
            for row in [
                "| Linux（GNU） | x86_64 | 2.00 MiB |",
                "| macOS | ARM64 | 3.00 MiB |",
                "| Windows | x86_64 | 4.00 MiB |",
                "| Windows | ARM64 | 5.00 MiB |",
            ]:
                self.assertIn(row, result.stdout)
            self.assertIn("<summary>二进制体积</summary>", result.stdout)

    def test_empty_directory_fails_without_a_partial_table(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            result = CliRunner().invoke(app, ["report", temporary])
            self.assertEqual(result.exit_code, 1)
            self.assertEqual(result.stdout, "")
            self.assertIn("没有找到", result.stderr)

    def test_corrupt_packages_fail_without_a_partial_table(self) -> None:
        for suffix in ["tar.xz", "zip"]:
            with (
                self.subTest(suffix=suffix),
                tempfile.TemporaryDirectory() as temporary,
            ):
                Path(temporary, f"kat-x86_64-pc-windows-msvc.{suffix}").write_bytes(
                    b"broken archive"
                )
                result = CliRunner().invoke(app, ["report", temporary])
                self.assertEqual(result.exit_code, 1)
                self.assertEqual(result.stdout, "")

    def test_missing_binary_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            with zipfile.ZipFile(
                Path(temporary, "kat-x86_64-pc-windows-msvc.zip"), "w"
            ) as archive:
                archive.writestr("README.txt", "missing executable")
            result = CliRunner().invoke(app, ["report", temporary])
            self.assertEqual(result.exit_code, 1)
            self.assertEqual(result.stdout, "")

    def test_help_and_directory_validation(self) -> None:
        result = CliRunner().invoke(app, ["report", "--help"])
        self.assertEqual(result.exit_code, 0)
        self.assertIn("ASSET_DIR", Text.from_ansi(result.stdout).plain)
        with tempfile.TemporaryDirectory() as temporary:
            result = CliRunner().invoke(
                app, ["report", str(Path(temporary, "missing"))]
            )
            self.assertEqual(result.exit_code, 2)


@app.command(name="self-test")
def self_test() -> None:
    """运行发布归档体积统计的回归测试。"""
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ReleaseSizeTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    raise typer.Exit(0 if result.wasSuccessful() else 1)


if __name__ == "__main__":
    app()
