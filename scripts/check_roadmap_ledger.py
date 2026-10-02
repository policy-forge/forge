#!/usr/bin/env python3
"""Check scope coverage, source wording and evidence references without Cargo/network."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sys


def check(root):
    ledger_path = root / 'docs/plans/2026-10-02-roadmap-requirement-ledger.json'
    ledger = json.loads(ledger_path.read_text())
    if ledger['format'] != 'forge.roadmap-ledger/1' or ledger['release_target'] != '2.0.0':
        raise ValueError('unsupported ledger format or release target')
    rows = ledger['rows']
    keys = [(row['prd'], row['id']) for row in rows]
    if len(set(keys)) != len(keys):
        raise ValueError('duplicate requirement identity')
    expected_prds = {f'{number:03d}' for number in range(55, 70)}
    if set(ledger['source_docs']) != expected_prds or {r['prd'] for r in rows} != expected_prds:
        raise ValueError('PRD 055–069 scope mismatch')
    pattern = r'^- \[([ xX])\] (\*\*([MS]-\d+)[\s\S]*?)(?=\n- \[[ xX]\]|\n### |\n## |\Z)'
    for prd, source in ledger['source_docs'].items():
        path = root / source['path']
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != source['sha256']:
            raise ValueError(f'{prd}: PRD changed; reconcile ledger explicitly')
        text = data.decode('utf-8')
        expected = {m[3]: (m[2].strip(), text[:m.start()].count('\n') + 1, m[1] != ' ')
                    for m in re.finditer(pattern, text, re.M)}
        actual = {r['id']: r for r in rows if r['prd'] == prd}
        if set(actual) != set(expected):
            raise ValueError(f'{prd}: missing/extra Must or Should requirements')
        for identity, row in actual.items():
            wording, line, checked = expected[identity]
            if (row['requirement'].strip(), row['source_line'], row['checkbox_checked']) != (wording, line, checked):
                raise ValueError(f'{prd}/{identity}: wording/source/checkbox drift')
            if row['source'] != source['path'] or row['status'] not in ledger['status_meanings']:
                raise ValueError(f'{prd}/{identity}: invalid source or status')
            if row['acceptance'] != 'open':
                raise ValueError(f'{prd}/{identity}: F01 snapshot cannot assert acceptance')
            if not row['remaining_gate'] or not row['owner'] or not row['evidence'] or not row['packages']:
                raise ValueError(f'{prd}/{identity}: missing evidence, gate, owner or package')
            for package in row['packages']:
                if package not in {f'F{n:02d}' for n in range(1, 23)}:
                    raise ValueError(f'{prd}/{identity}: invalid completion package')
            for reference in row['evidence']:
                target = re.sub(r':\d+(?:-\d+)?$', '', reference)
                path = Path(target)
                if path.is_absolute() or '..' in path.parts or not (root / path).exists():
                    raise ValueError(f'{prd}/{identity}: invalid evidence reference {reference}')
    if {g['prd'] for g in ledger['gate_registry']} != expected_prds:
        raise ValueError('missing per-PRD named gate coverage')
    for gate in ledger['gate_registry']:
        if gate['prd'] not in expected_prds or not gate['owner'] or not gate['decision']:
            raise ValueError('invalid gate owner/decision')
        if gate['status'] not in {'open', 'recorded-satisfied', 'satisfied-as-recorded', 'unverified'}:
            raise ValueError('invalid gate disposition state')
        for reference in gate['evidence']:
            if not (root / reference).exists():
                raise ValueError(f'missing gate evidence {reference}')
    for section in ledger['gate_source_sections']:
        lines = (root / section['source']).read_text().splitlines()
        if lines[section['source_line'] - 1] != '## ' + section['section']:
            raise ValueError('gate section source drift')
    markdown = (root / 'docs/plans/2026-10-02-roadmap-requirement-ledger.md').read_text()
    states = ('technical-evidence', 'partial', 'missing', 'conditional')
    for prd in expected_prds:
        counts = Counter(r['status'] for r in rows if r['prd'] == prd)
        summary = '| ' + prd + ' | ' + ' | '.join(str(counts[s]) for s in states) + ' |'
        if summary not in markdown:
            raise ValueError(f'{prd}: Markdown status totals differ')
    rendered = {}
    prd = None
    for line in markdown.splitlines():
        if line.startswith('### PRD '):
            prd = line.removeprefix('### PRD ')
        match = re.match(r'^\| \[([MS]-\d+): .*\]\(.*\) \| ([^|]+) \| ([^|]+) \|$', line)
        if match:
            key = (prd, match[1])
            if key in rendered:
                raise ValueError('duplicate Markdown requirement')
            rendered[key] = (match[2].strip(), match[3].strip().split(', '))
    if set(rendered) != set(keys):
        raise ValueError('Markdown requirement coverage differs')
    for row in rows:
        if rendered[row['prd'], row['id']] != (row['status'], row['packages']):
            raise ValueError('Markdown requirement state/package drift')
    counts = Counter(r['status'] for r in rows)
    expected_summary = f'{counts["technical-evidence"]} with technical evidence,\n{counts["partial"]} partial, {counts["missing"]} missing, and {counts["conditional"]} conditional.'
    if expected_summary not in markdown or f'**{len(rows)} requirements**' not in markdown:
        raise ValueError('Markdown overall counts differ')
    print(f'Ledger check: {len(rows)} unique Must/Should requirements; '
          f'{len(ledger["gate_registry"])} named gates; exact PRD text/hashes and evidence paths verified.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        check(args.root.resolve())
    except (ValueError, KeyError, IndexError, OSError, UnicodeError) as error:
        print(f'Invalid roadmap ledger: {error}', file=sys.stderr)
        sys.exit(2)
