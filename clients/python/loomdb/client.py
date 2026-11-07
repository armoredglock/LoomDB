"""
LoomDB Python Client
Zero-dependency, high-performance client driver for LoomDB.
"""

import json
import os
import shutil
import subprocess
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional, Union


class LoomError(Exception):
    """Exception raised for LoomDB query or execution errors."""
    pass


class LoomResult:
    """Represents a structured result set from a LoomDB query."""

    def __init__(self, data: Dict[str, Any]):
        self.status = data.get("status", "ok")
        self.columns = data.get("columns", [])
        self.rows = data.get("rows", [])
        self.count = data.get("count", len(self.rows))
        self.affected_rows = data.get("affected_rows", 0)
        self.elapsed_us = data.get("elapsed_us", 0)
        self.message = data.get("message", "")
        self.raw = data

    def to_dicts(self) -> List[Dict[str, Any]]:
        """Return the result rows as a list of dictionaries."""
        return self.rows

    def first(self) -> Optional[Dict[str, Any]]:
        """Return the first row of results, or None if empty."""
        return self.rows[0] if self.rows else None

    def __iter__(self):
        return iter(self.rows)

    def __len__(self):
        return self.count

    def __repr__(self):
        if self.rows:
            return f"<LoomResult rows={self.count} elapsed={self.elapsed_us}us>"
        return f"<LoomResult affected={self.affected_rows} elapsed={self.elapsed_us}us message='{self.message}'>"


class LoomDB:
    """
    Client for interacting with LoomDB.
    
    Can operate in:
      1. 'auto' mode (default): Connects to HTTP daemon if available, falls back to CLI.
      2. 'http' mode: Direct HTTP REST requests to LoomDB daemon (port 8484).
      3. 'cli' mode: Runs queries via the local `loom` command line tool with JSON formatting.
    """

    def __init__(
        self,
        mode: str = "auto",
        http_url: str = "http://localhost:8484",
        binary_path: Optional[str] = None,
        timeout: float = 5.0,
    ):
        self.http_url = http_url.rstrip("/")
        self.timeout = timeout
        self.binary_path = binary_path or self._find_binary()
        
        if mode not in ("auto", "http", "cli"):
            raise ValueError("mode must be one of: 'auto', 'http', 'cli'")
        
        self.mode = mode
        if self.mode == "auto":
            if self._check_http_alive():
                self._active_mode = "http"
            else:
                self._active_mode = "cli"
        else:
            self._active_mode = self.mode

    def _find_binary(self) -> str:
        # 1. In PATH
        in_path = shutil.which("loom") or shutil.which("loom.exe")
        if in_path:
            return in_path
        
        # 2. LocalAppData on Windows
        local_app = os.environ.get("LOCALAPPDATA", "")
        if local_app:
            cand = os.path.join(local_app, "LoomDB", "loom.exe")
            if os.path.exists(cand):
                return cand

        # 3. Development target path fallback
        repo_cand = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "target", "release", "loom.exe"))
        if os.path.exists(repo_cand):
            return repo_cand

        return "loom"

    def _check_http_alive(self) -> bool:
        try:
            req = urllib.request.Request(f"{self.http_url}/status", method="GET")
            with urllib.request.urlopen(req, timeout=0.8) as resp:
                return resp.status == 200
        except Exception:
            return False

    def query(self, sql: str) -> LoomResult:
        """
        Execute a SQL query against LoomDB and return a structured LoomResult.
        """
        if self._active_mode == "http":
            try:
                return self._query_http(sql)
            except Exception:
                if self.mode == "auto":
                    # Fallback to CLI if HTTP drops
                    self._active_mode = "cli"
                    return self._query_cli(sql)
                raise
        else:
            return self._query_cli(sql)

    def execute(self, sql: str) -> LoomResult:
        """Alias for query(), suitable for DDL/DML statements."""
        return self.query(sql)

    def _query_http(self, sql: str) -> LoomResult:
        url = f"{self.http_url}/query"
        payload = json.dumps({"sql": sql}).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=payload,
            headers={"Content-Type": "application/json", "Accept": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                body = resp.read().decode("utf-8")
                data = json.loads(body)
                if data.get("status") == "error":
                    raise LoomError(data.get("message", "Unknown database error"))
                return LoomResult(data)
        except urllib.error.HTTPError as e:
            try:
                err_data = json.loads(e.read().decode("utf-8"))
                raise LoomError(err_data.get("message", f"HTTP {e.code}"))
            except Exception:
                raise LoomError(f"HTTP Error {e.code}: {e.reason}")
        except Exception as e:
            raise LoomError(f"HTTP Connection failed: {e}")

    def _query_cli(self, sql: str) -> LoomResult:
        try:
            cmd = [self.binary_path, "query", sql, "--format", "json"]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=self.timeout,
                check=False,
            )
            stdout = result.stdout.strip()
            if not stdout:
                stderr = result.stderr.strip()
                raise LoomError(f"Loom CLI error: {stderr or 'No output received'}")

            data = json.loads(stdout)
            if data.get("status") == "error":
                raise LoomError(data.get("message", "Database error"))
            return LoomResult(data)
        except FileNotFoundError:
            raise LoomError(f"LoomDB binary not found at '{self.binary_path}'. Please install LoomDB or specify binary_path.")
        except json.JSONDecodeError:
            raise LoomError(f"Invalid JSON response from LoomDB: {result.stdout}")

    def tables(self) -> List[str]:
        """List all catalog tables in LoomDB."""
        if self._active_mode == "http":
            try:
                req = urllib.request.Request(f"{self.http_url}/tables", method="GET")
                with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                    data = json.loads(resp.read().decode("utf-8"))
                    return data.get("tables", [])
            except Exception:
                pass
        
        # CLI fallback
        res = self.query("SELECT * FROM _catalog_tables")
        return [r.get("name", "") for r in res.rows if "name" in r]

    def status(self) -> Dict[str, Any]:
        """Fetch engine telemetry and status metrics."""
        try:
            req = urllib.request.Request(f"{self.http_url}/status", method="GET")
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except Exception:
            return {
                "status": "ok",
                "mode": self._active_mode,
                "binary": self.binary_path,
                "version": "0.1.0",
            }

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass
