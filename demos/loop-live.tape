# Optional: record a fresh LIVE run (just mlpl-demo) with the pinned Ollama
# model. Takes minutes and depends on the model, so the committed demo is
# demos/loop.tape, which replays the saved transcript. Outputs go to
# assets/demo/loop-live.* and are ignored by git. Fixed sleep, no Wait.

Output assets/demo/loop-live.gif
Output assets/demo/loop-live.mp4

Set Shell bash
Set FontSize 13
Set Width 1000
Set Height 640
Set Padding 10
Set Theme "Builtin Dark"
Set Framerate 10
Set PlaybackSpeed 4.0

Type "LOOP_APPROVE=1 just mlpl-demo"
Sleep 800ms
Enter
Sleep 8m
