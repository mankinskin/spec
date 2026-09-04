---
description: "Create or update a draft spec entry from the slash-command text. Prefer spec-mcp tools and fall back to spec.exe when needed."
name: "spec"
argument-hint: "<your content>"
agent: "agent"
---

# Create or Update Draft Spec Entry

Create or update a draft spec entry from the user's current slash-command request.

Follow the canonical [spec authoring rules](../instructions/spec/spec-system.instructions.md), including component hierarchy and substantive child-spec requirements.

Workflow:
1. Treat the text typed after `/spec` as the source request.
2. Search existing specs first to avoid duplicates, build a component map, and make an explicit root-versus-child hierarchy decision.
3. Search existing tickets for the same work before deciding whether this should create a new spec or update an existing one.
4. Prefer `spec-mcp` tools such as `spec_search`, `spec_list`, `spec_tree`, `spec_create`, and `spec_update` when they are available.
5. If `spec-mcp` is unavailable, fall back to `./target/debug/spec.exe` and register `.spec/specs` with `spec.exe add-root .spec/specs --label default --json` if needed.
6. Infer a clear title, slug, component, and parent. Keep slugs lowercase, use `-` within segments, and `/` between segments.
7. Prefer updating a matching spec over creating a near-duplicate. If new independently addressable components require specs, create the shared root first and then each child with `--parent <root-id-or-slug>` in `draft` state.
8. Apply the canonical [code-first structure and relationship traceability rule](../instructions/spec/spec-system.instructions.md#code-first-structure-and-relationship-traceability): author children from target code and naming conventions through examples and reading-order links; author parents with their child-link list and `flowchart TD` subcomponent graph.
9. Ensure the spec body captures the intended system properties, explicit acceptance criteria, required evidence or traceability needed to evaluate implementation, and non-goals when obvious. Keep problem statements, current-state analysis, rollout sequencing, blockers, and implementation notes in related tickets unless the user explicitly asks for them in the spec.
10. When linking tickets in the spec or chat output, resolve the folder path per [AGENTS.md](../../AGENTS.md#clickable-reference-policy)'s Clickable Reference Policy: never synthesize it from a UUID, a store root, or an example path, and run a follow-up ticket-api command for the authoritative path if the first response omits it.
11. If the request clearly implies implementation work and the related ticket does not exist yet, create the needed ticket first or state explicitly that ticket creation is still required before implementation begins.
12. Follow [AGENTS.md](../../AGENTS.md#escalation-rules)'s escalation rule: if required details are still ambiguous after a focused search, ask one concise clarification rather than guessing.
13. Do not implement code or change unrelated files unless the user explicitly asks.

Response:
- created or updated spec slug and id
- chosen component and parent
- related tickets, rendered as markdown links per the Clickable Reference Policy in `AGENTS.md`, if any
- key assumptions
- duplicate candidates considered, if any
