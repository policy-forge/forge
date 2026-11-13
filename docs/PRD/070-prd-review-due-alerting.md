# 070-prd-review-due-alerting

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-11-12 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `feat/review-due-alerting`
**Created**: 2026-11-12
**Status**: Draft
**Input**: veans #65 (id 1472); PRD 058 review schedules; PRDs 057/065/068 notification exclusions

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will surface informational due/overdue review alerts derived from the existing
PRD 058 lifecycle schedule classification. Alerts are a deterministic, date-based
projection of the already-computed `due-soon`/`overdue` status: a CLI report and a
workspace owner-queue surface. This PRD scopes the notification channels that #65
asked for. It deliberately excludes email, webhooks, scheduled execution and
background sync, consistent with PRDs 057/065/068. Alerts never change review state,
imply approval, or mutate governed artifacts.

## Context

### Background :red_circle: `@human-required`

The lifecycle module already tracks per-policy `next_review_date`, `cadence_days` and
`due_soon_days` and derives `due-soon`/`overdue`/`approved-drifted` status plus an
owner-grouped review queue (PRD 058 M-9, S-2). What was missing is an explicit
*alerting* surface: a consumer had to read the full status/queue report and interpret
the schedule blockers themselves. #65 requested an alerting capability with pluggable
notification channels and noted that a scoped PRD must decide which channels are in
scope, because PRDs 057/065/068 deliberately exclude delivery today.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | `src/lifecycle/status.rs` already derives `due-soon`/`overdue` blockers from an explicit `--as-of` date with an inclusive boundary. | Alerts can borrow that classification without a second schedule engine or clock sampling. |
| Repository evidence | The workspace owner queue (`/api/v2/lifecycle/queue`) already returns `derived_status` and `blockers` per placement. | The UI can surface alerts from already-fetched rows with no wire-contract change. |
| Product boundary | PRDs 057/065/068 exclude email, webhooks, scheduled execution and background sync. | The in-scope channels are a deterministic local report and an in-session workspace surface, not delivery. |

## Problem Statement :red_circle: `@human-required`

Policy owners cannot see, at a glance, which reviews are due soon or overdue without
manually interpreting lifecycle status output. There is no dedicated alert report and
no workspace alert indicator, so due reviews are easy to miss even though the dates
and classification already exist.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Make due/overdue reviews legible. | A deterministic `forge lifecycle alerts` report lists every due-soon/overdue policy for an explicit date, ordered by urgency. |
| G-2 | Surface alerts where owners already work. | The workspace owner queue shows a per-placement review-alert badge and a per-page alert summary. |
| G-3 | Preserve authority and offline invariants. | Alerts change no recorded state, imply no approval, mutate no artifact, and fetch nothing outbound. |

## Non-Goals :red_circle: `@human-required`

- Email, webhook, chat or ticket delivery of alerts (PRDs 057/065/068 exclusions).
- Scheduled execution, background sync, or wall-clock-driven alerting.
- Any change to lifecycle state, approval inference, or governed artifact bytes.
- New crate dependencies; the projection reuses `chrono` and `serde` already in use.
- A workspace wire-contract change; alerts reuse the existing queue `derived_status`/`blockers`.

## Requirements

### Must Have (M) :red_circle: `@human-required`

- [x] **M-1 — Deterministic alert projection:** Derive alerts only from the existing `due-soon`/`overdue` schedule blockers for an explicit `--as-of` date; no clock sampling, no second schedule engine.
- [x] **M-2 — CLI report:** Provide `forge lifecycle alerts` with text/JSON output, the shared gate/exit contract, and safe atomic file output that cannot alias a record or artifact.
- [x] **M-3 — Workspace surface:** Show a review-alert badge per due-soon/overdue owner placement and a per-page alert summary in the dated owner queue, using already-fetched rows.
- [x] **M-4 — No authority change:** Alerts never mutate records or artifacts, never imply approval, and never alter recorded review state.
- [x] **M-5 — Boundary tests:** Cover due-today, overdue, future/outside-window, inclusive due-soon boundary, deterministic ordering, and drift-only (no schedule alert) cases.

### Should Have (S) :yellow_circle: `@human-review`

- [x] **S-1 — Days-until-review:** Each alert carries whole days from the explicit date to the review date (negative when overdue) for prioritization.

### Won't Have (W) — This release :red_circle: `@human-required`

- Notification delivery, scheduled/background execution, authentication, approval inference, or any outbound fetch.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A record whose next review equals the explicit date | `alerts` runs | One `due-soon` alert with `days_until_review` 0 |
| AC-2 | A record whose next review precedes the explicit date | `alerts` runs | One `overdue` alert with negative `days_until_review` |
| AC-3 | A record whose next review is outside the due-soon window | `alerts` runs | No alert; record bytes unchanged |
| AC-4 | An approved record with drift but a future review | `alerts` runs | No schedule alert (drift remains status evidence) |
| AC-5 | A dated workspace owner queue with due/overdue placements | The queue renders | Badges and a per-page alert summary appear; no write route is requested |

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-11-12 | In-scope channels are a deterministic CLI report and the workspace owner-queue surface; delivery is excluded. | PRDs 057/065/068 exclude email/webhooks/scheduling; the offline core must not fetch outbound. | Pluggable delivery connectors (deferred to PRD 065 follow-on). |
| 2026-11-12 | Alerts borrow the existing schedule classification rather than recompute dates. | One schedule engine keeps status, queue and alerts consistent and deterministic. | A separate alert date engine (duplicate logic, drift risk). |
| 2026-11-12 | Workspace alerts reuse fetched queue rows; no wire-contract change. | Avoids a contract version bump and keeps the read-only capture contract intact. | A new alerts endpoint (contract bump, larger surface). |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-11-12 | ReviewDueAlerts | Initial scoped PRD for due/overdue review alerting; CLI report and workspace surface implemented, delivery excluded |