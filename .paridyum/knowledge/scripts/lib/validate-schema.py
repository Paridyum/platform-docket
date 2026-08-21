#!/usr/bin/env python3
"""JSON Schema validation for knowledge artifacts and repository manifests.

    validate-schema.py frontmatter <schema.json> <file.md> [file.md ...]
    validate-schema.py document    <schema.json> <file.yaml> [file.yaml ...]

Exit 0 = all valid, 1 = at least one violation (printed), 2 = cannot validate.

WHY THIS EXISTS RATHER THAN `pip install jsonschema`
-----------------------------------------------------
`knowledge.frontmatter_valid` and `docs.manifest_schema_valid` were both declared as
schema validation and implemented as field-presence tests, so both accepted frontmatter
the schema rejects. The registry counted them as implemented; only their notes admitted
otherwise. That is the exact class of defect this repository keeps finding — a check
verifying less than its name claims.

Fixing it properly needs a validator. Adding a pip dependency to the gate that runs on
every commit means a network install in the hot path, and a gate that fails when a package
index is down is a gate people bypass. So this implements the subset of JSON Schema those
two schemas actually use, using only the standard library.

Supported: type (incl. unions), required, properties, additionalProperties:false, enum,
const, pattern, items, minItems, maxItems, uniqueItems, minLength, maxLength, minimum,
maximum, format:date. Anything else in a schema is IGNORED — see `unsupported()`, which
reports it rather than passing silently, because silently ignoring a constraint is how a
validator comes to verify less than its name claims.

PyYAML is required and is not vendored. If it is missing this exits 2 and the caller falls
back to its own weaker checks — loudly. It must never exit 0 when it could not validate.

DELIBERATE DIVERGENCE FROM THE REFERENCE IMPLEMENTATION
--------------------------------------------------------
Differential-tested against `jsonschema` over 16 cases; 15 agree. The one divergence is
`format: date`, which this enforces and `jsonschema` does not: in JSON Schema, `format` is
annotation-only unless a format-assertion vocabulary is enabled, so the reference
implementation accepts `created: 07-08-2026`. Here a malformed date is a real defect —
`knowledge.no_stale_proposed_artifacts` does arithmetic on `created` — so this is stricter
on purpose. Any other divergence found later is a bug in this file, not a policy.
"""

from __future__ import annotations

import json
import re
import sys
from datetime import date

try:
    import yaml
except ImportError:  # pragma: no cover - environment-dependent
    print("validate-schema: PyYAML not available; cannot validate", file=sys.stderr)
    sys.exit(2)


# Unquoted YAML dates become datetime.date, which is not a JSON Schema string. The
# schemas declare `format: date` on those fields, so keep them as written instead of
# letting the loader retype them and produce a spurious type error.
class _Loader(yaml.SafeLoader):
    pass


_Loader.yaml_implicit_resolvers = {
    k: [(t, r) for t, r in v if t != "tag:yaml.org,2002:timestamp"]
    for k, v in yaml.SafeLoader.yaml_implicit_resolvers.items()
}

KNOWN = {
    "type", "required", "properties", "additionalProperties", "enum", "const",
    "pattern", "items", "minItems", "maxItems", "uniqueItems", "minLength",
    "maxLength", "minimum", "maximum", "description", "default",
    "$schema", "$id", "title",
}
# `propertyNames` was listed as KNOWN and never implemented in validate(), so a schema
# using it would constrain nothing and be reported as fully supported. No schema uses it,
# so it is removed rather than implemented — the first one that does will now be told.
#
# `format` is the same defect and was found the same way, on 2026-08-21: it sat in KNOWN
# while validate() implements exactly one value, `date`. So `format: uri` in
# test-envelope.schema.json constrained nothing AND was reported as supported, which is
# the condition this list exists to prevent. It is now handled per value below rather than
# blanket-accepted — the keyword is recognised, and a value nobody implemented is named.
FORMATS_IMPLEMENTED = {"date"}

TYPES = {
    "string": str, "integer": int, "number": (int, float), "boolean": bool,
    "array": list, "object": dict, "null": type(None),
}


def unsupported(schema, path="", seen=None):
    """Report schema keywords this validator does not implement.

    A constraint that is silently ignored is worse than one that is absent, because the
    schema advertises it and nothing enforces it.
    """
    seen = seen if seen is not None else []
    if isinstance(schema, dict):
        for k, v in schema.items():
            if k == "format":
                # Recognised as a keyword, reported per unimplemented value. A schema
                # that advertises `format: uri` and gets no checking should say so.
                if v not in FORMATS_IMPLEMENTED:
                    seen.append(f"{path or '<root>'}: format: {v}")
            elif k not in KNOWN and not k.startswith("$"):
                seen.append(f"{path or '<root>'}: {k}")
            if k == "properties":
                for pk, pv in (v.items() if isinstance(v, dict) else []):
                    unsupported(pv, f"{path}.{pk}" if path else pk, seen)
            elif k == "items":
                unsupported(v, f"{path}[]", seen)
            elif k == "additionalProperties" and isinstance(v, dict):
                # The object form is a subschema. Not descending into it meant an unknown
                # keyword nested there was never reported.
                unsupported(v, f"{path}.*" if path else "*", seen)
    return seen


def _type_ok(value, spec):
    names = spec if isinstance(spec, list) else [spec]
    for n in names:
        py = TYPES.get(n)
        if py is None:
            continue
        # bool is a subclass of int in Python; JSON Schema treats them as distinct.
        if n in ("integer", "number") and isinstance(value, bool):
            continue
        if isinstance(value, py):
            return True
    return False


def validate(value, schema, path="$"):
    errs = []

    if "type" in schema and not _type_ok(value, schema["type"]):
        errs.append(f"{path}: expected type {schema['type']}, got {type(value).__name__}")
        return errs  # further checks would be noise

    if "const" in schema and value != schema["const"]:
        errs.append(f"{path}: must be {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        errs.append(f"{path}: {value!r} is not one of {schema['enum']}")

    if isinstance(value, str):
        if "pattern" in schema and not re.search(schema["pattern"], value):
            errs.append(f"{path}: {value!r} does not match {schema['pattern']}")
        if "minLength" in schema and len(value) < schema["minLength"]:
            errs.append(f"{path}: shorter than {schema['minLength']} characters")
        if "maxLength" in schema and len(value) > schema["maxLength"]:
            errs.append(f"{path}: longer than {schema['maxLength']} characters")
        if schema.get("format") == "date":
            try:
                date.fromisoformat(value)
            except (ValueError, TypeError):
                errs.append(f"{path}: {value!r} is not a YYYY-MM-DD date")

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            errs.append(f"{path}: below minimum {schema['minimum']}")
        if "maximum" in schema and value > schema["maximum"]:
            errs.append(f"{path}: above maximum {schema['maximum']}")

    if isinstance(value, list):
        if "minItems" in schema and len(value) < schema["minItems"]:
            errs.append(f"{path}: fewer than {schema['minItems']} items")
        if "maxItems" in schema and len(value) > schema["maxItems"]:
            errs.append(f"{path}: more than {schema['maxItems']} items")
        if schema.get("uniqueItems"):
            seen, dupes = [], []
            for it in value:
                (dupes if it in seen else seen).append(it)
            if dupes:
                errs.append(f"{path}: duplicate items {dupes}")
        if "items" in schema:
            for i, it in enumerate(value):
                errs += validate(it, schema["items"], f"{path}[{i}]")

    if isinstance(value, dict):
        for req in schema.get("required", []):
            if req not in value:
                errs.append(f"{path}: missing required property '{req}'")
        props = schema.get("properties", {})
        # additionalProperties has three forms and all three are now honoured. `false`
        # forbids unlisted keys; a schema object validates each unlisted key's VALUE
        # against it; absent permits anything. The object form was previously accepted
        # into the schema and then silently ignored here — `unsupported()` flags unknown
        # keywords, not unimplemented forms of known ones, so a constraint could be
        # advertised and enforce nothing with no warning anywhere. That is the precise
        # failure this validator exists to prevent, one level up.
        addl = schema.get("additionalProperties")
        if addl is False:
            for k in value:
                if k not in props:
                    errs.append(f"{path}: unexpected property '{k}'")
        for k, v in value.items():
            if k in props:
                errs += validate(v, props[k], f"{path}.{k}")
            elif isinstance(addl, dict):
                errs += validate(v, addl, f"{path}.{k}")

    return errs


def frontmatter(path):
    """Return the YAML frontmatter of a markdown file, or None if absent."""
    text = open(path, encoding="utf-8").read()
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    return yaml.load(parts[1], Loader=_Loader)


def main(argv):
    if len(argv) < 4:
        print(__doc__.strip().split("\n\n")[1], file=sys.stderr)
        return 2
    mode, schema_path, files = argv[1], argv[2], argv[3:]
    schema = json.load(open(schema_path, encoding="utf-8"))

    for gap in unsupported(schema):
        print(f"validate-schema: NOTE unsupported keyword, not enforced — {gap}", file=sys.stderr)

    failures = 0
    for f in files:
        try:
            doc = frontmatter(f) if mode == "frontmatter" else yaml.load(
                open(f, encoding="utf-8"), Loader=_Loader)
        except yaml.YAMLError as e:
            print(f"✘ {f}: unparseable YAML — {str(e).splitlines()[0]}")
            failures += 1
            continue
        if doc is None:
            print(f"✘ {f}: no frontmatter block")
            failures += 1
            continue
        for e in validate(doc, schema, "$"):
            print(f"✘ {f}: {e}")
            failures += 1
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
