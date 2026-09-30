# Feedback usability and visual-note review

## Scope and priorities

All four saved reports were reviewed locally. Their contents and filenames remain private. The engineering priorities and acceptance boundaries are authored in [feedback_review_plan.csv](../sheets/feedback_review_plan.csv). The 20 [manual cases](../sheets/feedback_editor_cases.csv) are Drew-owned and remain `not_run`.

1. Replace the four-field feedback form with one note. Preserve existing draft text, original captured context, local-only reports and explicit retargeting.
2. Embed a real Windows multiline editable surface in the same game window. Reuse OS editing and Text Services Framework rather than building a recognizer or launching an editor. Respect the user's existing speech hotkey.
3. Correct weapon animation names from original mounted metadata, author draw clips and avoid silent bind-pose success. Keep unsupported activity sequences explicit.
4. Restore useful category grouping and mounted original icons to creation cards, including meaningful hover context.
5. Keep earlier water and NoCollide fixes available for user comparison. Do not treat old reports as resolved without acceptance or claim a new water renderer was implemented here.

## Feedback workflow

Press F8 over the relevant UI or tool, click the single note field, then type or use the existing Windows dictation shortcut (normally Win+H). Save note locally. F8/Escape returns to the prior screen. No separate title, text tabs, external editor, microphone capture, speech SDK, agent process or automatic report upload is introduced.

Windows requires an actual focused editable control. On Windows the panel hosts `RICHEDIT50W` from system `msftedit.dll`, plain-text mode and explicit `SES_USECTF` text-services support. The DLL is loaded from System32 only. HWND operations stay on the main thread through a Bevy NonSend resource. Position follows the physical-pixel UI rectangle after layout. Only F8/Escape/F10 are forwarded to game navigation. Normal text, clipboard, selection, undo, IME and dictation stay with the control. Text changes are mirrored before save/navigation actions and persisted through the existing draft path. Focus is requested only on open or an explicit click, not stolen continuously from Windows speech UI.

Windows still owns its listening indicator, permissions, network speech processing and privacy settings. This is not a guarantee of invisible or offline dictation. End-to-end Win+H, GPU/child-window composition, DPI and focus behavior require Drew's checks. If control initialization fails, basic single-field keyboard editing remains with an explicit limitation. Other platforms keep basic typing only.

Version-3 reports add `body`, derive `title` from the first nonempty line, and retain compatibility fields and captured context. Four-field drafts migrate to a labeled combined note without deleting the original legacy draft or existing reports.

## Researched presentation changes

Ten original mounted weapon models were inspected as metadata only, supplying eleven enabled combat routes (flechette shares SMG). `source_weapons.csv` owns idle/fire/reload/draw names. Notable corrections: SMG `@fire01`, AR2 `@IR_idle/@IR_fire/@IR_reload/@IR_draw`, RPG `@idle1`, shotgun `@fire01` rather than its one-frame `@fire`. Draw gates primary firing until its clip ends; reload sampling follows the authored gameplay interval. Missing required clips reject the new visual route instead of silently reporting success in a bind pose.

This is not complete native animation sequencing. Shotgun shell-by-shell phases/pump, grenade activity variants, secondary attacks, sound events, camera motion, sway and detailed IK remain open. Metadata discovery is not animation-decoding or visual acceptance.

Creation cards reuse original `materials/<icon reference>` images, then existing model spawnicons where available. Absent images are labeled. Categories show counts and headings, entries sort within categories, and image/caption activation share a route. Original content is read-only and not redistributed. Pixel-perfect Derma layout and addon coverage remain open.

## Delivery rules

Compile the actual game and run the production catalog validator/exporter. Export a new complete workbook without replacing previous snapshots. Do not run tests, input automation, screenshots or dictation on Drew's behalf. Update the F7 cards with practical checks and honest limitations. Record actual build/export outcomes in VALIDATION.md, then commit and publish reviewed paths only.
