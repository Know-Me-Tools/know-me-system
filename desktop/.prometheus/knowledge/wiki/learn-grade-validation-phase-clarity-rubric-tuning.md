---
type: Reference
id: learn-grade-validation-phase-clarity-rubric-tuning
title: learn-grade validation phase clarity rubric tuning
tags:
- learn-grade
- grader-validation
- feynman-learning
- rubric-tuning
- clarity-scoring
- evaluation-dataset
- phase-status
sources:
- stdin
- manual:phase-learn-grader-validation
timestamp: 2026-07-16T20:44:19.090222+00:00
created_at: 2026-07-16T20:44:19.090222+00:00
updated_at: 2026-07-16T20:44:19.090222+00:00
revision: 0
---

## Phase Context

- **Phase:** `phase-learn-grader-validation`
- **Project:** unspecified
- **KBD root:** `$HOME/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-16T20:43:10Z`
- **Position:** `phase-learn-grader-validation`
- **Status:** execute in progress
- **Progress:** 6/7 changes complete; G-01/G-02/G-03/G-04 met
- **Latest completed change:** `change-lgv-006-tune-grader`
- **Next change:** `/kbd-apply change-lgv-007-regression-test-and-docs`

## Risk Being Addressed

`phase-learn-feynman` v1.4.0 shipped `learn-grade`, a sycophancy-corrected external grader that closes each Feynman loop, at only **60–70% assessed confidence**. The phase reflection identified this as the highest-severity open risk in the learn domain:

> A grader that misses misconceptions is worse than no grader — it provides false assurance.

The missing validation asset was an empirical evaluation dataset: the grader had not been tested against explanations with known, expert-labeled gaps.

## Phase Goals

- **G-01: Grader evaluation dataset**
  - Assemble 20+ Feynman explanations.
  - Cover at least 3 subject domains, e.g. STEM, humanities, and technical/programming.
  - Add expert-authored ground-truth annotations for:
    - misconceptions present,
    - misconceptions absent,
    - gold-standard score.
  - Store under:

    ```text
    skills/learn/learn-grade/references/eval-dataset/
    ```

  - Use a machine-readable schema: JSON or YAML per explanation.

- **G-02: Run `learn-grade` against the dataset**
  - Feed each explanation through the actual `learn-grade` skill/script path.
  - Do not use a mock grader.
  - Capture grader score and misconception list.
  - Diff grader output against ground truth.

## Completed Change: `change-lgv-006-tune-grader`

`kbd-apply` completed `change-lgv-006-tune-grader` with all **6/6 tasks** done and real tuning verified.

### Result

**G-04 met.** A systematic clarity-scoring bug in `learn-grade` was found, fixed, and validated.

### Root Cause

The original Step 3 clarity rubric asked whether the explanation was “understandable to the stated target level.” This wording was ambiguous. Grading agents consistently conflated conceptual correctness with prose clarity.

Observed behavior:

- Every factually flawed item tested, **8/8**, had its clarity score dragged down alongside its accuracy score.
- Clarity depression ranged from **0.18–0.35 points**.
- The failure mode made clarity partially redundant with accuracy instead of an independent dimension.

### Fix

`learn-grade/SKILL.md` was updated so Step 3 explicitly measures **prose quality independent of factual correctness**.

Design decision:

- **Accuracy** evaluates factual/conceptual correctness.
- **Clarity** evaluates prose understandability and communication quality.
- The two dimensions are intended to be orthogonal.

### Validation

Re-ran:

- the 8 affected factually flawed items,
- plus 1 control item.

Outcomes:

- All 8 flawed items’ clarity scores moved toward gold-standard values.
- Average clarity gap shrank from approximately **0.25** to approximately **0.03**.
- The control item moved away from its gold value.
  - Inspection found this was likely a gold-standard authoring bias.
  - The discrepancy was documented transparently rather than hidden.
- Accuracy and completeness metrics held steady.
- Misconception detection was unchanged, as expected from a clarity-rubric-only change.

## Metric Impact

| Metric | Before | After |
|---|---:|---:|
| Clarity Pearson r | 0.405 | **0.609** |
| Clarity Spearman | 0.379 | **0.625** |
| Clarity MAE | 0.160 | **0.088** |

The clarity correlation improved by roughly 50% while preserving other scoring dimensions.

## Artifacts and Commits

- New `TUNING-LOG.md` documents the full analysis.
- Fix commit pushed: `d495aa8`.
- KBD marker commit pushed: `68fa1dd`.

# Citations

1. stdin
2. manual:phase-learn-grader-validation
