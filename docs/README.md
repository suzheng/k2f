# K2F documentation

Public documentation for the K2F format and SDK.

**How to read this tree:**

- **[Specification](spec/k2f-v0.1.md)** — normative format contract (paths, fields, verify codes).
- **[Architecture](architecture/README.md)** — design rationale and engine internals for contributors.
- **[Instructions](instructions/)** — authoring operations for agents and hand-edited JSON.
- **[Agent skills](../skills/README.md)** — task-oriented playbooks that call the SDK/CLI.

## Specification

- [K2F format v0.1](spec/k2f-v0.1.md)

## Architecture

- [Overview](architecture/README.md)
- [Design goals and principles](architecture/design.md)
- [Codebase overview](architecture/codebase.md)
- [Layout engine](architecture/layout-engine.md)
- [Human editor (design proposal)](architecture/editor.md)

## Guides

- [Getting Started](guide/getting-started.md)
- [SDK themes](guide/themes.md)
- [Web viewer](guide/web-viewer.md)
- [Status & roadmap](guide/status.md)
- [Guide images](guide/images/README.md)
- [Desktop reader (in development)](../desktop/k2f_reader/README.md)

## Instructions (authoring)

- [Attribute reference](instructions/attribute_reference.md)
- [File operations](instructions/k2f_file_operations.md)
- [Content operations](instructions/content_operations.md)
- [Style operations](instructions/style_operations.md)
- [Semantic code blocks](instructions/semantic_code_blocks.md)
- [Agent profile v0](instructions/agent_v0.md) — SDK system-prompt dialect (not the format spec)

## Related

- [Agent skills](../skills/README.md)
- [Security policy](../SECURITY.md)
- [Contributing](../CONTRIBUTING.md)

---

*Internal implementation plans live outside this public repository (private brain), not under `docs/`.*
