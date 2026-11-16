#!/usr/bin/env python3
"""Expand a Hunspell .aff/.dic pair into a Rune word list: a count line, then one sorted word per line.

Usage: build_lang_dict.py <index.aff> <index.dic> <out.dict>
Words flagged FORBIDDENWORD or ONLYINCOMPOUND are dropped. Compound rules are ignored.
"""
import re
import sys
from collections import defaultdict
from pathlib import Path


def split_flags(s: str, long: bool) -> list[str]:
    return [s[i : i + 2] for i in range(0, len(s), 2)] if long else list(s)


def parse_aff(path: Path):
    long = False
    special: dict[str, str] = {}
    rules: dict[str, dict] = {}
    lines = path.read_text(encoding="utf-8").splitlines()
    for line in lines:
        p = line.split("#")[0].split()
        if len(p) >= 2 and p[0] == "FLAG":
            long = p[1] == "long"
        if len(p) >= 2 and p[0] in ("FORBIDDENWORD", "ONLYINCOMPOUND"):
            special[p[0]] = p[1]
    for line in lines:
        p = line.split("#")[0].split()
        if len(p) < 4 or p[0] not in ("PFX", "SFX"):
            continue
        if len(p) == 4 and p[3].isdigit():  # header line: kind flag cross count
            rules[p[1]] = {"kind": p[0], "cross": p[2] == "Y", "entries": []}
            continue
        flag = p[1]
        strip = "" if p[2] == "0" else p[2]
        add, _, cont = p[3].partition("/")
        add = "" if add == "0" else add
        cond = p[4] if len(p) > 4 else "."
        pat = (cond + "$") if p[0] == "SFX" else ("^" + cond)
        rules[flag]["entries"].append(
            (strip, add, re.compile(pat), split_flags(cont, long))
        )
    return long, special, rules


def apply(word: str, kind: str, strip: str, add: str, cond) -> str | None:
    if not cond.search(word):
        return None
    if kind == "SFX":
        if strip and not word.endswith(strip):
            return None
        return (word[: len(word) - len(strip)] if strip else word) + add
    if strip and not word.startswith(strip):
        return None
    return add + word[len(strip) :]


def expand(word: str, flags: list[str], rules: dict) -> set[str]:
    out = {word}
    sfx_forms: list[str] = []
    pfx = [f for f in flags if f in rules and rules[f]["kind"] == "PFX"]
    for f in flags:
        r = rules.get(f)
        if not r or r["kind"] != "SFX":
            continue
        for strip, add, cond, cont in r["entries"]:
            w = apply(word, "SFX", strip, add, cond)
            if w is None:
                continue
            out.add(w)
            if r["cross"]:
                sfx_forms.append(w)
            for c in cont:  # one level of continuation flags
                rc = rules.get(c)
                if rc and rc["kind"] == "SFX":
                    for s2, a2, c2, _ in rc["entries"]:
                        w2 = apply(w, "SFX", s2, a2, c2)
                        if w2:
                            out.add(w2)
    for f in pfx:
        r = rules[f]
        for strip, add, cond, _ in r["entries"]:
            for base in [word] + (sfx_forms if r["cross"] else []):
                w = apply(base, "PFX", strip, add, cond)
                if w:
                    out.add(w)
    return out


def main(aff: str, dic: str, out: str) -> None:
    long, special, rules = parse_aff(Path(aff))
    drop = {special[k] for k in special}
    words: set[str] = set()
    forbidden: set[str] = set()
    lines = Path(dic).read_text(encoding="utf-8").splitlines()[1:]
    for line in lines:
        line = line.split("\t")[0].strip()
        if not line:
            continue
        w, _, fl = line.partition("/")
        flags = split_flags(fl, long)
        if special.get("FORBIDDENWORD") in flags:
            forbidden.add(w)
            continue
        if special.get("ONLYINCOMPOUND") in flags:
            continue
        words |= expand(w, [f for f in flags if f not in drop], rules)
    words -= forbidden
    # Drop compound-only parts, and characters the Rune word-list format reserves.
    words = {w for w in words if w and not {w[0], w[-1]} & {"-"} and "/" not in w and "#" not in w}
    Path(out).write_text(f"{len(words)}\n" + "\n".join(sorted(words)) + "\n", encoding="utf-8")
    print(f"{len(words)} words", file=sys.stderr)


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:])
