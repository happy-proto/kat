#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "rich>=15.0.0",
#     "typer>=0.27.2",
# ]
# ///

from __future__ import annotations

import json
import os
import platform
import re
import struct
import time
from typing import Annotated, Any

import typer
from rich.console import Console
from rich.table import Table

app = typer.Typer(add_completion=False, help="读取当前 PTY 和终端实际报告的窗口几何。")

TERMINAL_REPORT_PATTERN = re.compile(rb"\x1b\[(4|6|8);(\d+);(\d+)t")
ENVIRONMENT_KEYS = (
    "TERM",
    "TERM_PROGRAM",
    "TERM_PROGRAM_VERSION",
    "COLORTERM",
    "COLUMNS",
    "LINES",
    "LANG",
    "LC_CTYPE",
)


def ioctl_geometry(fd: int) -> dict[str, int] | None:
    """读取 Unix PTY 的 winsize 结构。"""
    if os.name != "posix":
        return None

    import fcntl
    import termios

    try:
        rows, columns, pixel_width, pixel_height = struct.unpack(
            "HHHH",
            fcntl.ioctl(fd, termios.TIOCGWINSZ, bytes(8)),
        )
    except OSError:
        return None
    return {
        "columns": columns,
        "rows": rows,
        "pixel_width": pixel_width,
        "pixel_height": pixel_height,
    }


def stream_geometry(fd: int) -> dict[str, Any]:
    """读取一个标准流的 TTY 与行列信息。"""
    result: dict[str, Any] = {"isatty": os.isatty(fd)}
    if not result["isatty"]:
        return result
    try:
        size = os.get_terminal_size(fd)
        result["os_terminal_size"] = {
            "columns": size.columns,
            "rows": size.lines,
        }
    except OSError as error:
        result["os_terminal_size_error"] = str(error)
    result["ioctl"] = ioctl_geometry(fd)
    return result


def dev_tty_geometry() -> dict[str, Any] | None:
    """读取控制终端的 PTY winsize。"""
    if os.name != "posix":
        return None
    try:
        fd = os.open("/dev/tty", os.O_RDONLY | os.O_NOCTTY)
    except OSError:
        return None
    try:
        return {
            "path": "/dev/tty",
            "ioctl": ioctl_geometry(fd),
        }
    finally:
        os.close(fd)


def query_terminal_reports(timeout: float) -> dict[str, Any]:
    """通过标准 CSI 查询读取终端报告的 grid、窗口和 cell 像素。"""
    if os.name != "posix":
        return {
            "supported": False,
            "reason": "terminal queries require a Unix /dev/tty",
        }

    import select
    import termios
    import tty

    try:
        fd = os.open("/dev/tty", os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
    except OSError as error:
        return {"supported": False, "reason": str(error)}

    previous = None
    response = bytearray()
    try:
        previous = termios.tcgetattr(fd)
        tty.setraw(fd, termios.TCSANOW)
        os.write(fd, b"\x1b[14t\x1b[16t\x1b[18t")
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            remaining = deadline - time.monotonic()
            readable, _, _ = select.select([fd], [], [], remaining)
            if not readable:
                break
            try:
                chunk = os.read(fd, 4096)
            except BlockingIOError:
                continue
            if not chunk:
                break
            response.extend(chunk)
            report_kinds = {
                match.group(1) for match in TERMINAL_REPORT_PATTERN.finditer(response)
            }
            if report_kinds == {b"4", b"6", b"8"}:
                break
    except OSError as error:
        return {"supported": False, "reason": str(error)}
    finally:
        if previous is not None:
            termios.tcsetattr(fd, termios.TCSANOW, previous)
        os.close(fd)

    reports: dict[str, Any] = {"supported": True}
    for match in TERMINAL_REPORT_PATTERN.finditer(response):
        report_kind, first, second = match.groups()
        height = int(first)
        width = int(second)
        if report_kind == b"4":
            reports["text_area_pixels"] = {"width": width, "height": height}
        elif report_kind == b"6":
            reports["cell_pixels"] = {"width": width, "height": height}
        elif report_kind == b"8":
            reports["grid"] = {"columns": width, "rows": height}
    reports["response_hex"] = response.hex()
    if len(reports) == 2:
        reports["reason"] = "terminal did not return a recognized CSI size report"
    return reports


def collect_report(query_terminal: bool, timeout: float) -> dict[str, Any]:
    """汇总当前进程、PTY 与终端报告。"""
    report: dict[str, Any] = {
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
        },
        "streams": {
            "stdin": stream_geometry(0),
            "stdout": stream_geometry(1),
            "stderr": stream_geometry(2),
        },
        "controlling_tty": dev_tty_geometry(),
        "environment": {
            key: os.environ[key] for key in ENVIRONMENT_KEYS if key in os.environ
        },
        "multiplexer": {
            "tmux": "TMUX" in os.environ,
            "screen": "STY" in os.environ,
        },
    }
    report["terminal_reports"] = (
        query_terminal_reports(timeout)
        if query_terminal
        else {"supported": False, "reason": "disabled by --no-query-terminal"}
    )
    return report


def render_human(report: dict[str, Any]) -> None:
    """输出适合复制回聊天的人类可读表格。"""
    console = Console()
    console.print("[bold]Terminal geometry[/bold]")

    table = Table("source", "TTY", "columns", "rows", "pixel width", "pixel height")
    for name, value in report["streams"].items():
        ioctl = value.get("ioctl") or {}
        size = value.get("os_terminal_size") or {}
        table.add_row(
            name,
            str(value["isatty"]),
            str(ioctl.get("columns", size.get("columns", "-"))),
            str(ioctl.get("rows", size.get("rows", "-"))),
            str(ioctl.get("pixel_width", "-")),
            str(ioctl.get("pixel_height", "-")),
        )
    controlling = report.get("controlling_tty") or {}
    ioctl = controlling.get("ioctl") or {}
    table.add_row(
        "/dev/tty",
        str(bool(controlling)),
        str(ioctl.get("columns", "-")),
        str(ioctl.get("rows", "-")),
        str(ioctl.get("pixel_width", "-")),
        str(ioctl.get("pixel_height", "-")),
    )
    console.print(table)

    terminal_reports = report["terminal_reports"]
    report_table = Table("terminal report", "width", "height")
    for key in ("grid", "text_area_pixels", "cell_pixels"):
        value = terminal_reports.get(key)
        if value:
            report_table.add_row(
                key,
                str(value.get("columns", value.get("width", "-"))),
                str(value.get("rows", value.get("height", "-"))),
            )
    if report_table.row_count:
        console.print(report_table)
    else:
        console.print(
            f"[yellow]Terminal CSI report unavailable:[/yellow] "
            f"{terminal_reports.get('reason', 'no response')}"
        )

    context = report["environment"]
    if context:
        console.print("[bold]Environment[/bold]")
        for key, value in context.items():
            console.print(f"{key}={value}")
    multiplexer = report["multiplexer"]
    console.print(
        f"tmux={multiplexer['tmux']} screen={multiplexer['screen']}\n"
        "请在需要复现问题的同一个 terminal pane 尺寸下运行并复制完整输出。"
    )


@app.command()
def main(
    json_output: Annotated[
        bool,
        typer.Option("--json", help="输出稳定 JSON，便于原样复制或机器处理。"),
    ] = False,
    query_terminal: Annotated[
        bool,
        typer.Option(
            "--query-terminal/--no-query-terminal",
            help="查询终端主动报告的字符网格、文本区域像素和 cell 像素。",
        ),
    ] = True,
    timeout: Annotated[
        float,
        typer.Option(min=0.1, max=5.0, help="等待终端 CSI 响应的秒数。"),
    ] = 0.8,
) -> None:
    """读取当前 PTY 和终端窗口几何。"""
    report = collect_report(query_terminal, timeout)
    if json_output:
        print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    else:
        render_human(report)


if __name__ == "__main__":
    app()
