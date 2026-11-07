"""
LoomDB Python SDK
Lightweight, multi-modal database client for LoomDB.
Supports embedded CLI subshell and HTTP REST micro-daemon modes.
"""

from .client import LoomDB, LoomError, LoomResult

__version__ = "0.1.0"
__all__ = ["LoomDB", "LoomError", "LoomResult"]
