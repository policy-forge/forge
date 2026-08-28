# Reusable Policy Components

FORGE composes one policy from ordered, local Markdown components without a
template runtime, network access, nested includes, or environment interpolation.
Every source is hash-pinned. A successful composition writes assembled Markdown,
a lock, and a span-level provenance map as one coordinated operation.

## Component sidecars

A component source must be UTF-8 Markdown, begin with an ATX level-two heading,
and contain no level-one heading. Its adjacent, closed JSON sidecar uses
`forge.policy-component/1`:

```json
{
  "schema_version": "forge.policy-component/1",
  "component_key": "access-review",
  "version": "1.2.0",
  "title": "Access review",
  "owner": "security-governance",
  "status": "approved",
  "source": "access-review.md",
  "expected_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "parameters": [
    {
      "name": "owner-role",
      "type": "string",
      "required": true,
      "constraints": {
        "min_length": 2,
        "max_length": 100,
        "regex": "^[A-Za-z].+"
      }
    },
    {
      "name": "interval-days",
      "type": "integer",
      "default": 90,
      "constraints": { "minimum": 1, "maximum": 365 }
    }
  ]
}
```

Supported parameter types are `string`, `integer`, `boolean`, and
`string-list`. Constraints are type-checked: strings support length and regex,
integers support minimum/maximum, lists support item count plus per-item string
length/regex, and every type supports a typed `enum` array.

Scaffolding always creates a `draft` sidecar and never approves it:

```bash
forge policy component scaffold components/access-review.md \
  --component-key access-review --version 1.0.0 \
  --title "Access review" --owner security-governance \
  --output components/access-review.json

forge policy component check components/access-review.json
```

## One-pass placeholders

The only reserved token is `{{forge:param:<ascii-kebab-name>}}`. Placeholders
are allowed in ordinary paragraph and list text. They are rejected in headings,
fenced or inline code, link destinations, URLs, and raw HTML. Substituted strings
are escaped for Markdown structural punctuation and are never parsed a second
time. A value that looks like another FORGE placeholder remains escaped data.

Parameters are for non-sensitive organization values. Names containing
documented secret patterns such as `password`, `passphrase`, `secret`, `token`,
`api-key`, `private-key`, or `credential` are warned about and rejected. Values
are not read from environment variables or secret managers. Locks and
provenance contain parameter-value SHA-256 digests, not parameter values.

## Composition manifests

Paths are relative to the declared project root. Parent traversal, absolute and
drive-prefixed paths, alternate streams, symlink traversal, special files, and
input/output aliases are rejected. A component sidecar's `source` is relative
to the sidecar directory and must remain inside the project root.

```json
{
  "schema_version": "forge.policy-composition/1",
  "project_root": ".",
  "policy_key": "access-policy",
  "title": "Access and Identity Policy",
  "version": "2.0.0",
  "outputs": {
    "markdown": "build/access-policy.md",
    "lock": "build/access-policy.lock.json",
    "provenance": "build/access-policy.provenance.json"
  },
  "components": [
    {
      "instance_key": "quarterly-access-review",
      "component_manifest": "components/access-review.json",
      "parameters": {
        "owner-role": "IAM owners",
        "interval-days": 90
      }
    }
  ]
}
```

Use `check` for a side-effect-free validation. `--validate` additionally runs
the assembled bytes through FORGE's existing Markdown-to-OSCAL Catalog pipeline
without changing composition output.

```bash
forge policy compose check --manifest composition.json --validate
forge policy compose --manifest composition.json --validate
```

Exit status `2` means the manifest, pin, structure, parameter, path, rendering,
or provenance contract failed. Validation and pin failures happen before any
output is created or replaced.

## Update impact and OSCAL traceability

Reverse dependency analysis scans only manifests supplied by the caller. It is
read-only and reports both the expected and current component hash without
refreshing locks:

```bash
forge policy component impact --component-key access-review \
  --manifest policies/access.json --manifest policies/remote-access.json \
  --output access-review-impact.json
```

After converting assembled Markdown, pass the composition provenance file to
`forge trace` to append component file, source line, and instance origins to the
normal OSCAL trace report:

```bash
forge convert build/access-policy.md --strategy catalog --output build/catalog.json
forge trace build/catalog.json --source build/access-policy.md \
  --composition-provenance build/access-policy.provenance.json \
  --output build/trace.txt
```

Lifecycle labels are preserved as unauthenticated metadata. Composition does
not approve language, establish framework applicability or coverage, publish a
policy, or transition its lifecycle state.
