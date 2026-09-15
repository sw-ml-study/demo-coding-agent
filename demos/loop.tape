# The README's demo: mlplcode adds u:mul and a passing test to the MLPL
# example. `just replay` feeds the model replies recorded from a live
# qwen2.5-coder:7b run through the real loop, so the parser, the file
# writes, and the test run are live while the model is deterministic. No
# model server is needed and the run takes a few seconds.
#
# Fixed sleeps, no Wait: the convention from ../sw-os-ml. VHS's Wait+Screen
# matched the typed command line instead of program output and otherwise
# saw a stale screen, so every Wait-based attempt either returned before the
# run or timed out. Re-record deliberately with `just demo-tape`, which also
# optimises the GIF; only loop.gif is committed, loop.mp4 is regenerated.

Output assets/demo/loop.gif
Output assets/demo/loop.mp4

Set Shell bash
Set FontSize 13
Set Width 1000
Set Height 640
Set Padding 10
Set Theme "Builtin Dark"

# 10 fps: a terminal demo is flat colour and mostly static.
Set Framerate 10

Type "just replay"
Sleep 800ms
Enter
Sleep 14s
