# Development Loop Playbook

## Contents

- Scenario framing
- Verification path selection
- Evidence selection
- Loop recipes
- Exit rules

## Scenario Framing

- Write the scenario as one observable runtime outcome.
- Convert that outcome into 1-3 acceptance criteria.
- Keep each iteration focused on one failing or unknown criterion.

## Verification Path Selection

- Gameplay or device input behavior:
  Use `unity-playmode-testing` to enter Play Mode, send the narrowest input, and observe the result.
- UI interaction behavior:
  Use `unity-ui-automation` to locate the target element, inspect its state, interact once, and re-read state if needed.
- Visual end-state confirmation:
  Prefer a screenshot after the target state is visible.
- Timing or animation-sensitive behavior:
  Prefer a short video clip rather than many screenshots.
- Performance suspicion or explicit performance criteria:
  Use `unity-editor-tools` to check profiler status, capture briefly, and read only the needed metrics.

## Evidence Selection

- Start with the cheapest proof that can settle the current question.
- Prefer one evidence type per iteration unless the first signal is ambiguous.
- Good defaults:
  - state change or event path: logs or UI state read
  - layout or final visual state: screenshot
  - animation or timing: short video
  - frame time or spikes: profiler

## Loop Recipes

### Gameplay Input Loop

1. Lock the scenario and acceptance criteria.
2. Apply the smallest Unity-side change.
3. Enter Play Mode and wait until runtime is ready.
4. Send the exact keyboard, mouse, gamepad, or touch input needed.
5. Capture the minimum evidence that proves the reaction.
6. Read console output if the reaction is missing or unclear.
7. Record the iteration and decide whether to stop or continue.

### UI Flow Loop

1. Lock the UI scenario and acceptance criteria.
2. Apply the smallest Unity-side or binding-side change.
3. Enter Play Mode if the UI depends on runtime state.
4. Find the target element and inspect visibility or interactability.
5. Perform one interaction at a time.
6. Capture a screenshot when final UI state matters.
7. Read console output if clicks or value changes have no effect.
8. Record the iteration and choose the next hypothesis if needed.

### Regression Triage Loop

1. Reproduce the failing scenario with the narrowest path available.
2. Read the console before making changes.
3. Fix one suspected cause.
4. Rerun only the affected scenario path.
5. Stop when acceptance criteria are restored and diagnostics are clean enough for the scope of the change.

### Performance Suspicion Loop

1. Confirm the scenario is functionally correct first.
2. Check profiler status before starting capture.
3. Record a brief profiler session around the suspected hotspot.
4. Read only the metrics needed for the current hypothesis.
5. Make one focused change.
6. Rerun the same capture path and compare.

## Exit Rules

- Stop immediately when all acceptance criteria are satisfied by current evidence.
- Continue when at least one criterion is still failing and there is a concrete next hypothesis.
- Escalate scope only after a narrow loop fails to explain the behavior.
- Do not turn one scenario into a full exploratory session unless the user explicitly broadens the request.
