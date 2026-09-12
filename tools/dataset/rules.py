import json
from pathlib import Path
from typing import Any

DEFAULT_RULE_TABLE = Path(__file__).resolve().parents[2] / "specification" / "rule-table.json"


def load_rule_table(path: Path = DEFAULT_RULE_TABLE) -> dict[str, Any]:
    table = json.loads(path.read_text(encoding="utf-8"))
    if table.get("rule_set_id") != "prism.telemetry.p0" or table.get("version") != "1.0":
        raise ValueError("unsupported rule table")
    rules = table.get("rules")
    if not isinstance(rules, list) or [rule.get("precedence") for rule in rules] != list(range(1, len(rules) + 1)):
        raise ValueError("rule precedence must be contiguous")
    for rule in rules:
        if not all(key in rule for key in ("id", "field", "operator", "operand", "unit", "label", "severity", "route")):
            raise ValueError("rule is missing a required field")
        if not isinstance(rule["operand"], int):
            raise ValueError("rule operands must be integers")
    default = table.get("default")
    if not isinstance(default, dict) or not all(key in default for key in ("label", "severity", "route")):
        raise ValueError("rule default is invalid")
    return table


def _field_value(frame: Any, field: str) -> int | None:
    if field.startswith("sensor:"):
        sensor_id = int(field.split(":", 1)[1])
        for sensor in frame.sensors:
            if sensor.id == sensor_id:
                return sensor.value
        return None
    return getattr(frame, field)


def _matches(value: int | None, operator: str, operand: int) -> bool:
    if value is None:
        return False
    if operator == "eq":
        return value == operand
    if operator == "lt":
        return value < operand
    if operator == "gte":
        return value >= operand
    raise ValueError(f"unsupported operator: {operator}")


def evaluate_rules(frame: Any, rule_table: dict[str, Any]) -> dict[str, str]:
    for rule in rule_table["rules"]:
        if _matches(_field_value(frame, rule["field"]), rule["operator"], rule["operand"]):
            return {"rule_id": rule["id"], "label": rule["label"], "severity": rule["severity"], "route": rule["route"]}
    default = rule_table["default"]
    return {"rule_id": "DEFAULT", "label": default["label"], "severity": default["severity"], "route": default["route"]}
