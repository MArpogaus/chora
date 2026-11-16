# Tour v4: one usage scenario per look

The v3 tour showed the same doors (key list, helper chat, terminal) in every chapter, with long
waits. v4 gives each look one scenario for its own users (vision card) and shows each shared door
once. Every chapter runs 60 to 75 s, the handover 30 s, all together about 5 min.

Pace: at most 0.6 s between acts, typing about 12 characters a second, no still shot longer than
1.5 s (checked with `freezedetect` on the result), waits for animations are cut out of the video
(`Tour.cut`), the pointer moves at a fast, firm pace.

## 1 xp: Erika writes a letter (Erika 74, mouse only)
| Act | Shows | Only xp |
|-----|-------|---------|
| Start, Editor | the Start menu is the way to a program | Start menu, task button |
| Types a letter, one typo, saves | a plain editor in a window | XP window frame |
| Drags the window aside | the jelly wobble | wobbly windows |
| Paperclip: asks "How do I change the picture on my desktop?" | the helper in XP costume, with a question card for a risky act (Yes by mouse) | paperclip chat |
| Start, Control Panel, Desktop, picks a wallpaper | the answer found in Start | Control Panel cascade, wallpapers |

Shared doors shown here once: Start (program launcher) and the helper by mouse.

## 2 tiling: a hacker builds a project (keyboard-first)
| Act | Shows | Only tiling |
|-----|-------|-------------|
| Super+Return: a shell, `ls`, `nano` on a file | keys only | master and stack |
| Three more terminals, layouts by Super+Shift+space | re-tiling | layouts |
| Tag 2 for the build, back to tag 1 | tags | tags |
| Super+drag floats a window | wobble while floating | floating on top |
| Key list by Super+slash | every key, grouped | the key list for ricers |
| Screenshot of the rice (Print) | the notice | screenshot key |

## 3 mc: a hands-busy kitchen, and the sofa (voice and remote)
Light: recipe pages (own scenes), "next step" by voice (HUD line, stand-in voice agent), a question
answered on the HUD, a timer card (a notification; mc has no timer feature), Next by mouse, Left and
Right on the remote, Super+n for a larger page, a critical note when the timer is up.
Dark: the sofa in the evening: the plan, pages by the remote, Programs and chat doors by mouse.

## 4 construct: a demo watcher at the cube
Step back, turn by key (Ctrl+Alt+Right/Left) and by chip, F1 overlay, Return steps in, Alt+Escape
out, drag the cube by hand. Pulling a window off a face did not work with a scripted drag (see
the report); the window pull is shown in the handover.

## 5 handover: one window through all four looks
One editor window: xp, tiling, construct, xp, mc, xp by drags, then two key gives; window audit at
the end.
