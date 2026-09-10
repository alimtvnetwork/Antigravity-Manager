#!/usr/bin/env python3
"""
Fast Parallel Multi-Worker Local CI/CD Runner
==============================================
Runs repository quality gates concurrently via a controlled ThreadPoolExecutor worker group.
Provides flexible execution (parallel worker pool vs synchronous sequential),
smart log filtering (silent tick on success, full stack traces on failure),
file-based result logging, machine-readable JSON output, and CLI / AI programmatic interfaces.

Default Behavior:
- Runs in parallel using a worker pool bounded to CPU parallelism (capped at 8 to avoid I/O thrashing).
- Silent on success: prints a single tick line `✔ All passed. (21 gates in 2.34s)`.
- On failure: prints detailed error logs, exit codes, durations, and stack traces.

CLI Options:
- `--all-paths` / `--all-passed` / `--all` / `-a`: Detailed progress ticker, summary table, and full logs.
- `--failed` / `-f`: Show logs only for failed quality gates (default behavior).
- `--sync` / `--sequential` / `-s`: Run quality gates sequentially in a single thread.
- `--workers` / `-w` / `--concurrency`: Worker pool concurrency (default: CPU threads, capped at 8).
- `--output` / `-o` / `--file`: Save formatted execution report (or JSON) to a file.
- `--json`: Output results as machine-readable JSON for automated AI agent workflows.
- `--filter` / `-k`: Run only quality gates matching a substring (case-insensitive).
"""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass
from importlib import import_module
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Callable

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

sys.path.insert(0, str(Path(__file__).parent))
engine = import_module("02-shared-engine")

ExitCodeType = engine.ExitCodeType
DEFAULT_ENCODING = engine.DEFAULT_ENCODING
LINE_SEPARATOR = engine.LINE_SEPARATOR
DEFAULT_MAX_WORKERS = engine.DEFAULT_MAX_WORKERS
CI_JOBS_MATRIX = engine.CI_JOBS_MATRIX

DEFAULT_CONCURRENCY_WORKERS = engine.DEFAULT_CONCURRENCY_WORKERS


@dataclass
class JobResult:
    """Represents the execution outcome of an individual CI quality gate."""
    name: str
    is_success: bool
    output: str
    duration_sec: float
    return_code: int


@dataclass
class PipelineSummary:
    """Complete summary of a quality gate pipeline run."""
    total_jobs: int
    passed_count: int
    failed_count: int
    wall_duration_sec: float
    results: list[JobResult]
    has_failures: bool
    exit_code: int


CI_JOB_DEFAULT_ARGS: dict[str, list[str]] = {
    "Sequence & Title Check": ["01-prompts"],
    "Boolean Naming Check": ["04-code"],
    "Misspell Check": ["--staged"],
}


def resolve_job_command(job_name: str, command: list[str]) -> list[str]:
    """Resolves command with safe default targets if not explicitly specified."""
    if job_name in CI_JOB_DEFAULT_ARGS and len(command) <= 2:
        return [*command, *CI_JOB_DEFAULT_ARGS[job_name]]
    return list(command)


def execute_ci_job(job_name: str, command: list[str]) -> JobResult:
    """Executes a single validation check asynchronously and records output and duration."""
    start_time = time.perf_counter()
    effective_cmd = resolve_job_command(job_name, command)
    try:
        res = subprocess.run(
            effective_cmd,
            capture_output=True,
            text=True,
            encoding=DEFAULT_ENCODING,
            errors="replace"
        )
        duration = time.perf_counter() - start_time
        is_success = (res.returncode == 0)

        output_parts: list[str] = []
        stdout_clean = res.stdout.strip()
        if stdout_clean:
            output_parts.append(stdout_clean)
        stderr_clean = res.stderr.strip()
        if stderr_clean:
            output_parts.append(stderr_clean)

        combined_output = LINE_SEPARATOR.join(output_parts)
        return JobResult(
            name=job_name,
            is_success=is_success,
            output=combined_output,
            duration_sec=duration,
            return_code=res.returncode
        )
    except Exception as exc:
        duration = time.perf_counter() - start_time
        return JobResult(
            name=job_name,
            is_success=False,
            output=f"Failed to execute process: {exc}",
            duration_sec=duration,
            return_code=-1
        )


def execute_pipeline(
    target_jobs: dict[str, list[str]],
    worker_count: int = DEFAULT_CONCURRENCY_WORKERS,
    is_sync: bool = False,
    on_job_complete: Callable[[JobResult, int, int], None] | None = None
) -> PipelineSummary:
    """Executes target CI jobs in parallel or sequentially, returning structured summary."""
    effective_workers = 1 if is_sync else max(1, worker_count)
    start_wall_time = time.perf_counter()
    results: list[JobResult] = []

    if is_sync:
        for name, cmd in target_jobs.items():
            res = execute_ci_job(name, cmd)
            results.append(res)
            if on_job_complete:
                on_job_complete(res, len(results), len(target_jobs))
    else:
        with ThreadPoolExecutor(max_workers=effective_workers) as executor:
            future_map = {
                executor.submit(execute_ci_job, name, cmd): name
                for name, cmd in target_jobs.items()
            }
            for future in as_completed(future_map):
                res = future.result()
                results.append(res)
                if on_job_complete:
                    on_job_complete(res, len(results), len(target_jobs))

    total_wall_duration = time.perf_counter() - start_wall_time

    job_order = list(target_jobs.keys())
    results.sort(key=lambda r: job_order.index(r.name) if r.name in job_order else 999)

    passed_count = sum(1 for r in results if r.is_success)
    failed_count = sum(1 for r in results if not r.is_success)
    has_failures = (failed_count > 0)
    exit_code = ExitCodeType.VIOLATIONS_FOUND.value if has_failures else ExitCodeType.SUCCESS.value

    return PipelineSummary(
        total_jobs=len(target_jobs),
        passed_count=passed_count,
        failed_count=failed_count,
        wall_duration_sec=total_wall_duration,
        results=results,
        has_failures=has_failures,
        exit_code=exit_code
    )


def format_summary_text(summary: PipelineSummary, show_all: bool = False) -> str:
    """Formats human-readable text summary and error logs."""
    lines: list[str] = []

    if show_all:
        lines.append("======================= FINAL SUMMARY =======================")
        for r in summary.results:
            status_icon = "✅" if r.is_success else "❌"
            status_word = "PASSED" if r.is_success else "FAILED"
            lines.append(f"{status_icon} [{status_word}] {r.name:<40} ({r.duration_sec:.2f}s)")
        lines.append("------------------------------------------------------------")
        lines.append(f"Total Duration : {summary.wall_duration_sec:.2f}s")
        lines.append(f"Gates Passed   : {summary.passed_count}/{summary.total_jobs}")
        lines.append(f"Gates Failed   : {summary.failed_count}/{summary.total_jobs}")
        lines.append("------------------------------------------------------------")

        lines.append("\n===================== ALL QUALITY GATE LOGS =====================")
        for r in summary.results:
            status_word = "PASSED" if r.is_success else "FAILED"
            lines.append(f"\n--- [{status_word}] {r.name} ({r.duration_sec:.2f}s, exit code: {r.return_code}) ---")
            if r.output:
                lines.append(r.output)
            else:
                lines.append("(no output)")
            lines.append("-----------------------------------------------------------------")
    else:
        if summary.has_failures:
            lines.append("\n================== FAILED QUALITY GATE LOGS ==================")
            for r in summary.results:
                if not r.is_success:
                    lines.append(f"\n❌ FAILED: {r.name} (exit code: {r.return_code}, duration: {r.duration_sec:.2f}s)")
                    lines.append(f"--- {r.name} LOG ---")
                    if r.output:
                        lines.append(r.output)
                    else:
                        lines.append("(no output)")
                    lines.append(f"--- END {r.name} LOG ---")
            lines.append("-----------------------------------------------------------------")
            lines.append(f"Total Duration : {summary.wall_duration_sec:.2f}s")
            lines.append(f"Gates Passed   : {summary.passed_count}/{summary.total_jobs}")
            lines.append(f"Gates Failed   : {summary.failed_count}/{summary.total_jobs}")
            lines.append("------------------------------------------------------------")
            lines.append(f"\n❌ Pipeline failed: {summary.failed_count} quality gate(s) reported violations.")

    return LINE_SEPARATOR.join(lines)


def write_output_file(output_path: str, content: str) -> None:
    """Safely writes report content to output file, creating directories if needed."""
    p = Path(output_path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding=DEFAULT_ENCODING)


def run_pipeline(
    jobs: dict[str, list[str]] | None = None,
    max_workers: int | None = None,
    show_all: bool = False,
    is_sync: bool = False,
    output_file: str | None = None,
    as_json: bool = False,
    filter_pattern: str | None = None,
) -> int:
    """Dispatches CI jobs using worker group or sequential runner, with selective reporting."""
    all_jobs = jobs or CI_JOBS_MATRIX

    if filter_pattern:
        pattern_lower = filter_pattern.lower()
        target_jobs = {k: v for k, v in all_jobs.items() if pattern_lower in k.lower()}
        if not target_jobs:
            print(f"⚠️ No CI quality gates matched filter: '{filter_pattern}'")
            return ExitCodeType.TOOL_ERROR.value
    else:
        target_jobs = all_jobs

    worker_count = max_workers or min(len(target_jobs), DEFAULT_CONCURRENCY_WORKERS)
    if is_sync:
        worker_count = 1

    if not as_json:
        if show_all:
            concurrency_label = "Sequential (1 worker)" if is_sync else f"{worker_count} parallel workers"
            print("================================================================")
            print("           PARALLEL LOCAL CI/CD QUALITY GATE RUNNER             ")
            print("================================================================")
            print(f"🚀 Execution Mode          : {concurrency_label}")
            print(f"📋 Total Enqueued Gates    : {len(target_jobs)}")
            print("🔍 Display Mode            : SHOW ALL INFORMATION")
            print("----------------------------------------------------------------\n")

    def ticker_callback(res: JobResult, current: int, total: int):
        if not as_json:
            if show_all:
                status_icon = "✅" if res.is_success else "❌"
                status_label = "PASS" if res.is_success else "FAIL"
                print(f"[{current:2d}/{total:2d}] {status_icon} [{status_label}] {res.name} ({res.duration_sec:.2f}s)")

    summary = execute_pipeline(
        target_jobs=target_jobs,
        worker_count=worker_count,
        is_sync=is_sync,
        on_job_complete=ticker_callback
    )

    if as_json:
        payload = {
            "total_jobs": summary.total_jobs,
            "passed_count": summary.passed_count,
            "failed_count": summary.failed_count,
            "wall_duration_sec": round(summary.wall_duration_sec, 2),
            "has_failures": summary.has_failures,
            "exit_code": summary.exit_code,
            "gates": [asdict(r) for r in summary.results]
        }
        json_content = json.dumps(payload, indent=2, ensure_ascii=False)
        if output_file:
            write_output_file(output_file, json_content)
            print(f"📄 JSON results saved to: {output_file}")
        else:
            print(json_content)
        return summary.exit_code

    if output_file:
        full_file_report = format_summary_text(summary, show_all=True)
        write_output_file(output_file, full_file_report)
        print(f"📄 Execution report saved to: {output_file}")

    if summary.has_failures:
        failure_text = format_summary_text(summary, show_all=False)
        print(failure_text)
        return summary.exit_code

    if show_all:
        all_text = format_summary_text(summary, show_all=True)
        print(all_text)
        print("\n🎉 All quality gates passed successfully! Codebase is 100% green.")
    else:
        print(f"✔ All passed. ({summary.passed_count} gates in {summary.wall_duration_sec:.2f}s)")

    return summary.exit_code


def parse_arguments() -> argparse.Namespace:
    """Parses command-line arguments for CI runner."""
    parser = argparse.ArgumentParser(
        description="Fast Multi-Worker Local CI/CD Runner with parallel worker pool and flexible reporting.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Default: run all gates in parallel; quiet on success (tick), detailed logs on failure:
  python 03-ai-scripts/06-cicd-local-runner.py

  # Show all information (ticker, summary table, full logs for all gates):
  python 03-ai-scripts/06-cicd-local-runner.py --all-paths
  python 03-ai-scripts/06-cicd-local-runner.py --all-passed
  python 03-ai-scripts/06-cicd-local-runner.py --all

  # Run sequentially (synchronous mode, 1 worker):
  python 03-ai-scripts/06-cicd-local-runner.py --sync

  # Custom worker concurrency:
  python 03-ai-scripts/06-cicd-local-runner.py --workers 4

  # Save report to a file:
  python 03-ai-scripts/06-cicd-local-runner.py --output tmp/cicd-report.txt

  # Output machine-readable JSON:
  python 03-ai-scripts/06-cicd-local-runner.py --json -o tmp/cicd-report.json

  # Filter specific gate:
  python 03-ai-scripts/06-cicd-local-runner.py --filter "Go Base"
        """
    )
    parser.add_argument(
        "--all", "--all-paths", "--all-passed", "-a",
        action="store_true",
        dest="show_all",
        help="Show detailed information and logs for all quality gates (both passed and failed)."
    )
    parser.add_argument(
        "--failed", "-f",
        action="store_true",
        dest="show_failed",
        help="Show logs only for failed quality gates (default behavior)."
    )
    parser.add_argument(
        "--sync", "--sequential", "-s",
        action="store_true",
        dest="is_sync",
        help="Execute quality gates sequentially (1 worker) instead of in parallel."
    )
    parser.add_argument(
        "--workers", "-w", "--concurrency",
        type=int,
        default=None,
        dest="workers",
        help="Number of concurrent worker threads (default: CPU threads, capped at 8 to prevent I/O thrashing)."
    )
    parser.add_argument(
        "--output", "-o", "--file",
        type=str,
        default=None,
        dest="output_file",
        help="Save execution results and report to the specified file path."
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="as_json",
        help="Output results as machine-readable JSON (useful for automated AI agents)."
    )
    parser.add_argument(
        "--filter", "-k",
        type=str,
        default=None,
        dest="filter",
        help="Filter jobs matching substring (case-insensitive)."
    )
    return parser.parse_args()


def main():
    args = parse_arguments()
    exit_code = run_pipeline(
        max_workers=args.workers,
        show_all=args.show_all,
        is_sync=args.is_sync,
        output_file=args.output_file,
        as_json=args.as_json,
        filter_pattern=args.filter
    )
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
