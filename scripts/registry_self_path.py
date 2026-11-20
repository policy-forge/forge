"""Validate a registry `self` path before it is appended to the registry origin.

Exit 0 when the value is a plain absolute path that cannot change the URL's
authority: single leading '/', no '@', no backslash, no whitespace or control
characters. Any other value would let the registry redirect the download host
(e.g. '@evil.example/x' puts registry.oscal.io in userinfo).
"""
import sys

value = sys.argv[1] if len(sys.argv) == 2 else ""
ok = (
    value.startswith("/")
    and not value.startswith("//")
    and "@" not in value
    and "\\" not in value
    and all(0x20 < ord(ch) < 0x7F for ch in value)
)
sys.exit(0 if ok else 1)
