<!-- aligned-structure:v1 -->

# Summary

The `StateTransition` component surfaces the spec state machine as an interactive stepper in the editor UI.

## Behavior Story

The `StateTransition` component surfaces the spec state machine as an interactive stepper in the editor UI.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# spec-editor/state-machine

The `StateTransition` component surfaces the spec state machine as an interactive
stepper in the editor UI.

## State Machine

```
draft → reviewed → approved → implemented → verified
                                              ↓
                                     (terminal: done)

Any state → cancelled
Any state → deprecated
```

`reviewed` and `approved` must appear in `history.ndjson` before `verified` is
reachable (enforced by `spec-api` schema `required_states`).

## Stepper Display

Renders the ordered state list as a horizontal stepper:

```
[draft] → [reviewed] → [approved] → [implemented] → [verified]
   ●            ○             ○              ○               ○
```

- Filled circle = current or past state (from `history.ndjson`).
- The active state is highlighted with the component's accent colour.
- Future states are greyed out.

## Advance Button

- Shows the next valid state transition: "Advance to **reviewed**".
- Clicking triggers pre-flight:
  1. Calls `GET /api/specs/:id/health` — displays any issues.
  2. If issues exist and `required_states` demands this step, the button is disabled
     and issues are listed inline.
  3. If pre-flight passes, shows a confirmation modal: "Advance from `draft` to
     `reviewed`?"
  4. On confirm: `POST /api/specs/:id/advance` (new endpoint added by `spec-editor`
     backend — wraps `spec update --state`).
  5. On success: stepper updates; SSE broadcasts `spec.updated` to other clients.

## Archive Actions

A separate "Archive" dropdown button offers:
- "Mark Deprecated" → `POST /api/specs/:id/advance` with `state=deprecated`.
- "Cancel" → `POST /api/specs/:id/advance` with `state=cancelled`.

Both require a short reason text (free-form input in the dropdown).

## Health Gate

Pre-flight checks that block advancement:

| Transition | Blocking conditions |
|---|---|
| `draft → reviewed` | body.md is empty OR fewer than 1 CodeRef |
| `reviewed → approved` | any CodeRef fails `validate_refs` |
| `approved → implemented` | no `acceptance.md` section present |
| `implemented → verified` | all CodeRefs must be present and validate |

Blocking conditions are advisory in v1 (shown as warnings, not hard errors) to allow
flexibility.  A future schema flag `strict_health_gate = true` will make them hard
blockers.
