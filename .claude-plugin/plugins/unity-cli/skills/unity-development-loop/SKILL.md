---
name: unity-development-loop
description: Implement and validate Unity runtime scenarios through repeated fix-run-verify loops with Unity C# edits, Play Mode checks, UI interaction, evidence capture, and console or profiler review. Use when the user asks to implement and verify gameplay, input, or UI flows, reproduce and fix a runtime bug, or iterate on a Unity-side change until acceptance criteria are met. Do not use for `.inputactions` authoring, read-only inspection, or Rust CLI-only work.
compatibility: Requires unity-cli connected to a Unity Editor that supports Play Mode control, runtime input simulation, UI automation, and editor diagnostics.
allowed-tools: Bash, Read, Grep, Glob
metadata:
  author: akiojin
  version: 0.1.0
  category: workflow
---

# Unity Development Loop

Run Unity-side development as short, acceptance-driven loops.
This skill chooses the smallest next change, the narrowest runtime check, and the lightest evidence set that can prove or disprove the current hypothesis.

Read `references/development-loop-playbook.md` when you need scenario-specific loop recipes or evidence selection rules.

## Use When

- The user wants to implement and verify a Unity gameplay, input, or UI scenario.
- The task needs a repeatable loop of Unity-side code changes, Play Mode execution, evidence capture, and runtime confirmation.
- The user asks to iterate on a runtime bug until acceptance criteria are satisfied.
- The request needs screenshots, short video, console inspection, or profiler data as part of Unity runtime validation.

## Do Not Use When

- The task is authoring `.inputactions` assets instead of validating runtime behavior. Use `unity-input-system`.
- The task is read-only code or scene investigation with no planned change. Use the relevant inspection skill.
- The task is Rust CLI-only or does not require Unity runtime validation.
- The task is scene, prefab, or asset authoring without a runtime validation loop.

## Delegation Map

- Unity C# implementation: `unity-csharp-edit`
- Play Mode control, runtime input, screenshots, video: `unity-playmode-testing`
- UI locate, inspect, interact sequences: `unity-ui-automation`
- Editor state, console, profiler: `unity-editor-tools`

## Core Loop

Copy this checklist and update it as the loop progresses:

```text
Development Loop:
- [ ] Step 1: Lock scenario and acceptance criteria
- [ ] Step 2: Choose the smallest next change
- [ ] Step 3: Apply the change through the right lower-level skill
- [ ] Step 4: Run the narrowest runtime check
- [ ] Step 5: Capture only the evidence needed
- [ ] Step 6: Inspect logs or profiler if the result is unclear
- [ ] Step 7: Record the iteration result
- [ ] Step 8: Stop if accepted; otherwise define the next hypothesis
```

### Step 1: Lock Scenario And Acceptance Criteria

- Restate the scenario as observable runtime behavior.
- Turn it into 1-3 concrete acceptance checks before editing or running anything.
- If the request mixes unrelated behaviors, split it into separate loops.

### Step 2: Choose The Smallest Next Change

- Change one behavior at a time.
- Prefer the smallest Unity-side edit that can move the scenario forward.
- If the next step is implementation, delegate to `unity-csharp-edit`.

### Step 3: Apply The Change Through The Right Skill

- Use exactly one lower-level skill as the primary executor for the current step.
- Pull in a second skill only when the scenario genuinely crosses boundaries, such as UI interaction during Play Mode.
- Do not duplicate lower-level instructions already defined by those skills.

### Step 4: Run The Narrowest Runtime Check

- Verify only the behavior touched by the current iteration.
- Prefer targeted Play Mode input or a short UI interaction over a broad rerun.
- Use scenario-specific recipes from `references/development-loop-playbook.md` when selection is unclear.

### Step 5: Capture Adaptive Evidence

- Use logs or state reads for non-visual behavior.
- Use a screenshot for visible end-state confirmation.
- Use short video only when motion or timing matters.
- Use profiler capture only for performance requirements or credible performance suspicion.

### Step 6: Inspect Runtime Diagnostics

- Read the console after failures or ambiguous results.
- Check editor state before retrying if the runtime may not be ready.
- Clear logs only after reading them when a clean rerun is needed.

### Step 7: Record The Iteration

Use this structure after every loop:

```markdown
Iteration N
- Scenario:
- Acceptance Criteria:
- Change Made:
- Execution:
- Evidence:
- Observations:
- Result: pass | partial | fail
- Next Action:
```

### Step 8: Stop Or Continue

- Stop when every acceptance criterion is satisfied by the current evidence.
- Continue only with one concrete next hypothesis.
- Avoid broad rewrites, full-suite reruns, and full-evidence capture unless the latest iteration changed shared foundations.

## Examples

- "Implement a jump input tweak, run it in Play Mode, and keep iterating until the jump timing feels correct."
- "Fix this settings panel flow, click through the UI, capture proof, and stop when the final state matches the spec."
- "Reproduce the runtime bug, inspect logs, make the smallest fix, and rerun only the affected path."

## Common Mistakes

- Starting Play Mode before defining acceptance criteria.
- Capturing screenshots, video, logs, and profiler data on every iteration.
- Using the profiler for a purely functional failure with no performance signal.
- Rerunning the whole scenario after a tiny localized change.
