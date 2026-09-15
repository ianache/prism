import re
from typing import Any


CASE_NAMES = ("tcp smoke", "http smoke", "tcp 100000", "http 100000")
SECRET_PATTERN = re.compile(
    r"BEGIN [^-]*PRIVATE KEY|PRISM_AUTH_TOKEN|TOKEN=|PASSWORD=|CREDENTIAL",
    re.IGNORECASE,
)


def _contains_secret(value: Any, path: str = "bundle") -> str | None:
    if isinstance(value, dict):
        for key, child in value.items():
            if SECRET_PATTERN.search(str(key)):
                return f"secret-like field at {path}.{key}"
            found = _contains_secret(child, f"{path}.{key}")
            if found:
                return found
    elif isinstance(value, list):
        for index, child in enumerate(value):
            found = _contains_secret(child, f"{path}[{index}]")
            if found:
                return found
    elif isinstance(value, str) and SECRET_PATTERN.search(value):
        return f"secret-like value at {path}"
    return None


def validate_bundle(bundle: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(bundle, dict):
        return ["bundle must be an object"]
    secret = _contains_secret(bundle)
    if secret:
        errors.append(secret)
    for field in ("schema_version", "source", "commit", "digest", "cases"):
        if field not in bundle:
            errors.append(f"missing field: {field}")
    if bundle.get("schema_version") != "s24.v1":
        errors.append("schema_version must be s24.v1")
    if bundle.get("source") not in ("ci", "wsl2"):
        errors.append("source must be ci or wsl2")
    cases = bundle.get("cases")
    if not isinstance(cases, list) or [case.get("name") for case in cases if isinstance(case, dict)] != list(CASE_NAMES):
        errors.append("cases must contain tcp smoke, http smoke, tcp 100000, http 100000 in order")
        return errors
    for case in cases:
        if not isinstance(case, dict):
            errors.append("case must be an object")
            continue
        name = case["name"]
        status = case.get("status")
        if status not in ("PASS", "FAIL", "NO EJECUTADA"):
            errors.append(f"{name}: invalid status")
        if status == "NO EJECUTADA" and not case.get("reason"):
            errors.append(f"{name}: reason required for NO EJECUTADA")
        if status == "FAIL" and not case.get("diagnostic"):
            errors.append(f"{name}: diagnostic required for FAIL")
        if status != "PASS":
            continue
        if case.get("protocol") not in ("tcp", "http"):
            errors.append(f"{name}: protocol required")
        if case.get("name") in ("tcp smoke", "http smoke"):
            if case.get("frames_sent") != 3 or case.get("frames_ok") != 3 or case.get("synthetic_cycle") is not False:
                errors.append(f"{name}: PASS smoke requires 3/3 and synthetic_cycle=false")
        elif case.get("name") == "tcp 100000":
            if case.get("frames_sent") != 100000 or case.get("frames_ok") != 100000:
                errors.append(f"{name}: comparable TCP gate requires exactly 100000/100000")
        elif case.get("frames_sent") != 100000 or case.get("frames_ok") != 100000:
            errors.append(f"{name}: HTTP measurement requires 100000/100000")
    return errors
