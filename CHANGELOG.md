## v0.9.0-rc.4 (2026-11-17)

### BREAKING CHANGE

- output.configure and output.reserve need the
capability methods.privileged on a confined connection (a client socket
or a choractl mcp). The MCP tools configure_output and reserve_space
fail with code 4 until the client name has the grant, for example
`choractl grant voice-mcp methods.privileged`.
- chora-xp.yaml with the key assistant is rejected; move
assistant.enabled to chora-assistant.yaml as enabled and remove the key.

### Feat

- **tiling**: restyle the bar in the look of the windows
- **tiling**: wobbly windows on by default
- **hud**: a sentence takes a size, and mc reads it at 19 px
- **gate**: run the steps after clippy in parallel with --jobs
- **gate**: run the red-first rule as a commit-msg hook
- **test**: run the scripts of the e2e job in a merge gate step
- **dist**: add chora-report, an anonymised bundle for a report
- **voice**: write the log to voice.log as well
- **chora**: write the log to chora.log as well
- **scene**: add rotating log files under the state home
- **assistant**: say why a list in a margin is empty
- **assistant**: say No helper where no helper exists
- **construct**: a corner strip with the clock and the doors
- **construct**: the keys of every look in the walk and in a pane
- **mc**: the helper door always shows a state word
- **mc**: add the doors row at the foot of the column
- **mc**: bind the shared keys and list them
- **xp**: the shared keys and doors of every look
- **tiling**: show the word of the helper door in the bar
- **tiling**: show the key list on Super+F1, name the volume keys
- **kit**: the state word of a helper door without a helper
- **xp**: add Notepad, the plain text editor, to the start menu
- **xp**: default chrome is XP Normal at 96 DPI
- **xp**: CHORA_XP_ASSETS folder; scale 2 in the sway e2e
- **tiling**: focus glow in the accent, captures on the wallpaper
- **tiling**: helper surfaces wear Ink and Paper
- **construct**: faces of one manager carry a number on the label
- **construct**: the help names the managers as people read them
- **construct**: the walk keys go in pane steps
- **construct**: Super slash shows the help, F1 stays in Walk
- **construct**: command surfaces tint 0.92 over a stronger scrim
- **construct**: labels hang at the bottom edge, capsules are short
- **construct**: the frame of a pane is the buffer of its window
- **construct**: a face under the pointer lights its edge
- **construct**: the cube rests 14 degrees off square
- **construct**: the cube follows the hand while it is dragged
- **construct**: a face that starts or fails says so on its face
- **construct**: a terminal from the walk opens beside the panes
- **construct**: a notice is at most 520 wide
- **construct**: no grey slab, no app id, a refused give says why
- **construct**: walk hints show what applies, Super T included
- **construct**: a second program opens flown in beside the first
- **construct**: a start card on the empty floor
- **tiling**: the key list has a short mouse line under its header
- **tiling**: Light look and Dark look in Apps switch the theme
- **mc**: a click on the head of a page opens the page list
- **mc**: the footer names the previous page and a click goes back
- **xp**: helper tray icon; Help window in taskbar and Alt+Tab
- **xp**: balloon with close box, Send, white field, two-line header
- **xp**: the question card in XP form, group box and push buttons
- **xp**: start menu 420 wide, filled left column, 14 px rows
- **xp**: the taskbar is 36 px, task buttons 180 by 28
- **xp**: all chrome text is 14 px, menus and dialogs sized to it
- **hud**: the speech balloon fades in 160 ms
- **hud**: idle_stop grey keeps Stop in place; xp uses it
- **xp**: the whole Control Panel row opens after 400 ms
- **xp**: 80 by 88 icon grid, paperclip 8 pixels in, black balloons
- **xp**: tooltips, icon-only speaker, 24 pixel desktop buttons
- **xp**: caption buttons 24 by 24 and a 14 pixel bold title
- **xp**: Super opens Start, Super+d shows the desktop
- **xp**: start menu has chat and help rows and plain subtitles
- **xp**: Super+F1 and F1 open Help and Support with the live keys
- **tests**: ci-local replays the CI jobs on this machine
- **tests**: hud-audit in the tiling, mc and xp gates
- **tests**: merge-gate worker mode, red-first, pre-commit range
- **assistant**: hover mark on card buttons, Stop and list rows
- **assistant**: set the card in a page as the mc card
- **mc**: the chat in the margin column, notifications fold
- **mc**: state and helpers at the top of the margin column
- **mc**: Super+a chat, Super+Home agenda, Super+Return terminal
- **mc**: muted and amber of the light theme at 4.5:1 on the paper
- **mc**: say Nothing new. when there are no notifications
- **mc**: margin column of 26 % in whole lines with a hairline
- **mc**: scale 19 on a 28 px grid and a 14 px label style
- **mc**: clickable key rows, helpers and rows with a focus mark
- **mc**: key list in the margin column, Super+F1 for all keys
- **tiling**: the theme follows the system, with Paper and a gate
- **tiling**: screenshot keys and a power menu
- **tiling**: shadows and motion, with animations false
- **tiling**: key list on Super+/, layout chip, Super+space apps
- **tiling**: gaps, rounded corners and the focus border
- **tiling**: swallow every unbound Super chord
- **tiling**: bar doors, apps, tags, layout word, keys, power
- **tiling**: bar as three floating islands in Ink
- **demo**: capture the marble floor of the construct
- **construct**: render the marble floor finer on a GPU
- **construct**: lay a floor of polished marble under the panes
- **gate**: run one gate at a time with a lock
- **assistant**: move the ring of the island in busy states
- **assistant**: dim the island while the camera is in a pane
- **construct**: name the face of the cube stood back
- **construct**: open the cube with Super+C instead of a chip
- **construct**: blur the space behind the glass of the HUD
- blur the frame behind a rect with a backdrop
- **chat**: let the field tell its height to the view
- **xp**: make the chat the one balloon of the paperclip
- **construct**: show a notice as one line under the island
- **construct**: draw the HUD of the construct as glass
- **assistant**: draw the chat of a glass folder as the glass column
- **chat**: stop the helper with Ctrl+. in the field
- **assistant**: size the glass chat to the output, 400 to 480
- **assistant**: draw the voice island and the glass card
- **kit**: share the glass tokens of the construct HUD
- **mc,construct**: name the theme to the HUD host
- **hud**: colour a folder by the theme of its window manager
- **assistant**: show the state word beside the figure
- **assistant**: name the voice state in the Chat row
- **construct**: lift a window clear of the neighbour face
- **construct**: pull a window from the stood-back cube
- **mc,construct**: leave the voice HUD to the host by default
- **flavors**: start chora-assistant in tiling and construct
- **construct**: window managers in panes
- **construct**: add the chip OPEN CUBE as a visible way in
- **xp**: list the shared keys in the Control Panel
- **xp**: add a tray speaker, Sound and the volume keys
- **tiling**: change the volume by the media keys and label it
- **kit**: add the volume of the default output through wpctl
- **assistant**: looks where nothing covers anything
- **assistant**: helper rows, the chat slot and the look folders
- **construct**: read true and false of voice_hud as own and host
- **kit**: slot, margin, panel colours and figure in hud.yaml
- **construct**: hand the voice HUD to the host, a glass chat folder
- **mc**: hand the voice HUD to the host and open the chat drawer
- **xp**: make the assistant assets a HUD folder
- **mc**: a drag gives a window to another window manager
- **core**: a click on an empty claim chooses its manager
- **construct**: hover edge, drag to turn, voice words, plain Escape
- **xp**: a taskbar of a virtual output lists its own windows
- **construct**: windows float off the faces of the cube
- **kit**: a swung sail bows far
- **core**: a recorded frame shows an arrow, not the mark
- **core**: record frames with a stepped scene clock
- **demo**: record the walk and the swing as GIF and MP4
- **demo**: a construct scene with generated, colourful content
- **construct**: a swung pane bows and its frame follows
- **core**: record frames with a stepped scene clock
- **construct**: step back from a cube face and step in
- **kit**: the modes of a view on a cube
- **core**: render.stats counts frames, damage and passes by output
- **dist**: ship a minimal sandbox for the coding agents
- **core**: a flag turns on the test mode of drag gives
- **xp**: place a moved window at a point, send the drag speed
- **construct**: give a pane dropped over a face back to the cube
- **core**: a test mode where an injected press gives its drag
- **construct**: take a window that comes with its drag
- **xp**: give a window that a drag pulls out of its face
- **xp**: no cube of its own on virtual outputs
- **xp**: windows on several virtual outputs
- **construct**: host a cube of virtual outputs with one xp
- **kit**: the cube geometry
- **core**: no window strands on a removed virtual output
- **core**: a press through a pane stays in its output
- **core**: a pane forwards the pointer into its output
- **construct**: wobble a dragged pane in 3D
- **construct**: direct input on the 3D base (K5)
- **kit**: give the drag with window.give
- **core**: give the drag of a window with the window
- **core**: a shader node reads its virtual output
- **core**: virtual outputs with an owner on every backend
- **kit**: wobbly windows in 3D with a depth for the mesh
- **core**: name the focused window that a dropped layer holds
- **chat**: add the launcher entry Chat with a helper
- **tiling**: open the chat with Super+a
- **assistant**: host the chat session and draw its view
- window managers ignore injected keys unless a test says so
- one give key action for tiling, xp, mc and construct
- **kit**: drive wobbly and lamp from chora_kit::effects
- **assistant**: start the chat field on a pipe without a socket
- **chat**: add chora-chat, the opener and the chat field
- **test**: add 3d-bench.sh and the live mode of the probe
- **core**: step plane textures and cut units at the near plane
- **core**: keep layer textures and split orders between frames
- **assistant**: pick the HUD folder by context
- **assistant**: host one HUD folder with the question card
- **core**: let popups and client drags see every surface in a plane
- **voice**: add the epoch of an agent start to agent.read
- **kit**: add the chat model with sanitizer, wrap and held input
- **core**: carry the review of step 2 into contexts
- **core**: sort, split and hit the planes of 3D contexts
- **scene**: add the perspective of 3D contexts
- **draw**: load a HUD folder with confined pictures and fonts
- **core**: draw and hit planes outside a 3D context
- **scene**: stop walks at planes and add the hit list
- **kit**: add the HUD folder, word table and question card models
- **voice**: add the chat methods and a transcript per agent
- **kit**: add a notification client and a process check
- **core**: mark a pointer press of input.inject as injected
- **core**: name a confined caller in method.called
- **xp**: start chora-notify and chora-assistant in the XP flavor
- **assistant**: move the assistant from chora-xp to chora-assistant
- **notify**: move notifications from chora-xp to chora-notify
- **draw**: add the bubble outline and the corners of an output
- **draw**: move theme, text and icons of chora-xp to chora-draw
- **hooks**: run the guard test when a guard changes
- **scene**: add inverse, plane_inverse and decompose to Transform
- **persona**: keep the captures and the log with PERSONA_KEEP
- **persona**: start the companions of the flavor in the session
- **render**: count offscreen renders, elements and damaged frames
- **plugins**: add skeletons of chora-notify and chora-assistant
- **hooks**: add commit-message-length guard with its test
- **hooks**: add no-session-files guard with its test
- **hooks**: add no-inline-comments guard with its test
- **mc**: mark agent.set and notify.push privileged
- **choractl**: pass the privileged flag in the MCP tool meta
- **core**: send the privileged flag in methods.list

### Fix

- **construct**: a placed pane is saved and survives a restart
- show the screenshot notice with no bus, add Screenshot to power
- open Notepad only in an editor with a window
- refuse unshowable security changes and own HUD questions
- sanitize and guard the voice confirm question
- say a Notepad fallback opened only when its window shows
- Notepad falls back to another editor and says why
- **construct**: the strip has a Helper word that opens the chat
- **construct**: a placed pane stays in view when the cube steps back
- **mc**: a window dragged over the mc output shows above the page
- compare every shipped plugin data dir in packaging check
- wait for killed clients in gate-0.8-4 and gate-0.7-4
- wait for killed clients before removing the runtime dir
- install the tiling HUD in the Arch package
- **words**: the stuck line names the state of the handler
- **assistant**: a view that closes hides the field every time
- **assistant**: a question does not hold the Enter of the list
- **tiling**: the bar word is the one door of the chat in tiling
- **tiling**: no tooltip on the doors of the bar
- **tiling**: show the result of a screenshot without a notifier
- **xp**: expect the Question head while the card waits
- **xp**: draw the card with its hint in one frame
- **xp**: set the key words of the question card under its buttons
- **core**: a key counted as real keeps the pointer gate
- **construct**: a stood-back cube turns, shows keys and closes
- **core**: a key grab held back by the pointer is swallowed
- **mc**: merge mc r1 and r2, panel keys, column, empty head
- **mc**: a critical notice is a card, and a full column sheds notes
- **mc**: a column that came back shows no empty head
- **mc**: Mute stands beside the state word, not under the host's
- **mc**: the page widens when Super+n hides the column
- **mc**: the page keeps the keyboard when a helper fills a panel
- **security**: merge secfix-a r2, icon dir and FIFO checks
- read sender icons once, render a private copy
- decode only small PNG files for a path a sender names
- **core**: merge multilook r1, pane keys and cube drag
- **construct**: a drag beside an open cube does not look around
- **core**: only managers of virtual outputs give way for keys
- **core**: a key grab goes to the manager under the pointer
- **xp**: merge xp-polish r1, paperclip back, fields on grid
- **xp**: the tray speaker shows the volume level as a bar
- **xp**: open Writer, mousepad or gedit before a terminal editor
- **assistant**: keep the idle figure to the xp form
- **xp**: put the edit boxes on the grid of 4 px
- **assistant**: show the figure while no voice agent runs
- **voice**: merge secfix-b r1, API keys out of agent env
- **voice**: require a confirm rule for run_program
- **voice**: keep API keys from the coder and the agents
- **render**: draw a mesh element only inside its damage
- **tiling**: trap the bar signals by name before the fork
- **tiling**: start waybar with the push signals ignored
- **safety**: excuse the longest wait of one thread, not the sum
- **safety**: count the wait of all threads as host load, not as cost
- refuse an injected press on every button of the chat
- carry the injected flag on every input event of a person
- a notification card takes no injected press
- tiling ignores an injected press or key
- xp ignores an injected press, release, move or key
- an injected pointer does not drive the construct look
- an injected key does not turn the mc page
- an injected key does not drive the construct walk
- **assistant**: lay out only the header lines that show
- **assistant**: keep two header lines of room in the plain chat
- **voice**: keep the keys line of a long HUD question
- **voice**: make earcon_volume a security key
- **kit**: answer settings.set and reload as the plugin shows them
- **voice**: keep spoken notes out of the answers of the LLM
- **voice**: redact the JSON-RPC lines of the agent log
- **voice**: localize a reloaded file in the running language
- **xp**: give each test theme a cache folder of its own
- let a click of input.inject do nothing in mc
- make notify.clear privileged
- refuse panel.set on the agents and the overview scene
- ignore an injected key in the window list of the construct
- **guards**: pass kill tools in any order after need or command -v
- **gate**: close an allow in the runtime grep before a comment
- **gate**: read command substitutions in the no-foreign-kill hook
- **gate**: open an allow in the runtime grep only on an attribute
- **hook**: pass a fix on the base branch itself with a note
- **gate**: let an allow cover only its item in the runtime grep
- **gate**: take a live chora-xp beside its zombie in xp.sh
- **gate**: read commands, not text, in the no-foreign-kill hook
- **gate**: refuse in the red-first hook only what it can prove
- **gate**: run 2d-guard alone before the parallel steps
- **lint**: compare the two clippy configs both ways
- **gate**: judge the programs of a gate by pid, not by name
- **gate**: never block a valid commit in the red-first hook
- cap the chat header at two lines in every look
- wrap the no-helper header of the chat instead of cutting it
- **draw**: check a foreign file as a file of a HUD folder
- **assistant**: bound the lines and the events of the field
- **construct**: take only a left press of the person on the list
- **construct**: cap the ends that the fit of a text tries
- **mc**: ignore injected presses on hits and agent columns
- **mc**: make hud.set and hud.say privileged
- **mc**: refuse content.set on the contents that mc writes
- cap rmcp at warn in the filter of the voice log
- open the log files without following symlinks, fix old modes
- chora-report hides block YAML secrets and follows symlinks
- only the holder of NAME.lock writes and moves a log
- **test**: drop the font skips from the e2e allowlist
- **test**: draw the text of gates 0.8-9, 0.8-10 as a free client
- **test**: scope injected keys, check the camera refusal in agent.sh
- **test**: count the skips of the e2e sequence as failures
- **test**: fail gate 0.7-6 in CI when xkb lacks the fkeys option
- **test**: skip gate 0.7-6 without the fkeys option of xkb
- **test**: move the camera of gate 0.8-2 as a free client
- **test**: bring tests/agent.sh to the rules of the core
- **mc**: let only the side of the key list give way to all keys
- pick after a text of the list only by a click or a number
- end the text of the list with the list
- **construct**: show the doors stood back from a cube
- name the give keys in the key lists of tiling and mc
- **assistant**: open the only helper and send a text after a pick
- **assistant**: show the notice of the xp balloon in two lines
- **mc**: end the header of an app where the column begins
- **chora**: keep key presses of smithay out of the trace log
- **chora**: log each refused request with its client and code
- **voice**: keep spoken and typed text out of every log level
- **mc**: keep a binding over new settings
- **tiling**: keep a binding over a change of the grabs
- **xp**: keep the other keys over a change of the cube keys
- **words**: a fault handler that waits for the disk goes on
- route plugin output list and subscribe through chora-kit
- **tiling**: restate the focus opacity when a border is restyled
- require fuzzel and fontconfig, recommend DejaVu in packages
- **assistant**: say No helper whenever no helper runs
- **kit**: list the shared entries in the launcher of xp and tiling
- **mc**: say once that the chat does not open
- **kit**: wrap the launcher with the shared entries in one place
- **keys**: one cap and one wording for the volume keys
- **construct**: hide the doors strip in a pane
- **mc**: leave the words of no helper to the HUD host
- **construct**: hide the doors capsule while a cube is open
- **construct**: free F1 and open the key list on Super+F1
- **mc**: keep the long keys of the full list off their words
- **test**: wait for the depth frame in e2e step 21
- **test**: accept the filtered green of a plane in e2e step 21
- **test**: read the rects of the surfaces in the e2e overlap check
- **notify**: cap the body before the layout of the card
- **mc**: require the label style in the settings
- **xp**: clear a pending row hover when its cascade closes
- **tiling**: drop selection-radius from the fuzzel config
- **construct**: keep a binding over a change of the grabs
- **mc**: hand the keyboard back unless another client took it
- **mc**: hand the keyboard back only to whom it belonged
- **mc**: hand the keyboard back when the chat closes
- **mc**: a page turn leaves the keyboard with an open chat
- **mc**: a page beside other window managers takes no keyboard
- **mc**: the state line holds the word Mute
- **mc**: a critical notification says Urgent and leaves on a click
- **mc**: plain arrows turn the page and Escape goes back
- **core**: draw the injection mark in front of the cursor
- **voice**: log words over a reply and tool arguments at debug
- **core**: draw the injection mark in front of every client node
- gate output.configure and output.reserve on methods.privileged
- **voice**: keep the API key variables for agents and the harness
- **voice**: drop the URL from every request error of the LLM
- **core**: count no injected input as activity for idle-notify
- **voice**: log utterances, replies and coding tasks at debug
- **voice**: create the history file and the agent log with 0600
- **voice**: create a recording with the modes 0700 and 0600
- **voice**: name a server in errors without userinfo and query
- **voice**: drop the echo of a question before the yes/no match
- **voice**: keep the key rule with the question it belongs to
- **voice**: ignore a confirm key in the first second of a question
- **voice**: hide typed text and cut the arguments in the journal
- **voice**: start the children without the API key variables
- **choractl**: refuse a view save that loses the saved file
- **choractl**: count only absolute config directories
- **kit**: keep the values of a broken settings file out of its error
- **core**: log the length of a text without glyphs, not the text
- **core**: keep the injection mark on its output
- **core**: keep quit, reset and revoke off the lock screen
- **core**: bound what an export and a call cost the exporter
- **core**: cap the grabs of a connection at 256
- **core**: refuse headless outputs for a confined connection
- **core**: keep attach of a client socket off screen:*
- **core**: give injected keys a keyboard source of their own
- **core**: lay out no text with a line under 0.01 pixels
- **core**: refuse a key name with a NUL before xkbcommon
- **xp**: say Nothing found in the Run box
- **tiling**: sort the Apps list and say when nothing is found
- **render**: index the input choice in hit_guard for the msrv build
- **xp**: cascade new windows in the order in which they came
- **construct**: time an event without a time by the clock
- **core**: send the time of an input as a 32-bit count
- **core**: refresh the pointer with the time of its last input
- **construct**: take the speed of a flick from the input times
- **construct**: a click during a step of the cube counts
- **construct**: a deaf window manager blocks one wait in a round
- keep one display open and give each test its own directory
- **words**: the alarm of a fault ends only a stuck handler
- **persona**: start assistant, voice, notify and lock stubs
- **xp**: menu header gap, generic icon, tight chat, neutral copy
- **xp**: or between chords, one header row, Turn Off and Run shots
- **xp**: 16 px title and task icons, whole art, readable speaker
- **assistant**: refit the balloon in the step that shows a question
- **xp**: audit icon sizes and clipped text; task, cascade, Cancel
- **xp**: one-line field, grey path strip, tail at the box
- **xp**: the notify balloon stands left of the paperclip
- **xp**: the paperclip picture starts at its head
- **xp**: inactive caption buttons are 24 px and the close stays red
- **xp**: the tray icons are 16 pixel glyphs on an even grid
- **xp**: the start menu user tile is one square with an equal ring
- **tiling**: crisp icons, Inter card, chat clear of card and toast
- **tiling**: chips beside the chat, tinted scrim, more captures
- **tiling**: status chips and orbs in the tokens, audit cells
- **tiling**: one surface, icons, key list grid, toast placement
- **chat**: Close at the right edge, a round input field
- **tiling**: clear the card from the tile border
- **tiling**: keep the neutral dot and sentence as they were
- **tiling**: hairline of the helper surfaces is line
- **construct**: whole panes, a mirror with a body, one capsule rim
- **construct**: a deaf window manager blocks one face, not all
- **construct**: neighbour labels show, soft mirror edges, gate
- **construct**: step in dims to 0.6, failed face copy per the look
- **construct**: neighbour labels, 22 degree rest, a mirror that ends
- **construct**: only a pane in front of the cube gives way (M1)
- **construct**: neighbour panes keep their colours in the walk (M6)
- **construct**: panes give way to the cube stood back (M1)
- **construct**: dark glass for hints over a program, rim inside (M5)
- **construct**: pad the text of the first terminal 12 by 12 (M7)
- **construct**: no corner chip in use mode, a Step back hint (M2)
- **construct**: hide the pane label while a cube is open (M3)
- **construct**: regrab after leaving a flight, stop a stuck walk key
- **construct**: one 10 s manager limit, plain card foot, docs match
- **construct**: the next face stands at the right of the cube
- **construct**: Super slash also hides the help
- **construct**: say that a window manager does not run
- **mc**: dark Listening at 7:1, the empty agenda names the key
- **mc**: all text colors at 7:1, a voice hint on the empty agenda
- **mc**: no widow in the card, a drawn close mark, muted text at 7:1
- **mc**: helper rows on the label column, state line on the grid
- **chat**: the field, Stop and the card stand on the column grid
- **mc**: one baseline grid, one column top, one label width
- **mc**: helpers get a head, tracking leaves the edge, chat on 28
- **mc**: rows share baselines, notifications breathe
- **mc**: the terminal page keeps the column and its hairline
- **mc**: all-keys list clears the column, foot reads its colors
- **mc**: bars span the capitals of their text, gaps even
- **tiling**: look notice, Mouse line counts for the panel width
- **tiling**: shorten the mouse line so it stays inside the key list
- **assistant**: the paperclip hops in whole pixels, never eased
- **xp**: Run opens fuzzel in the look of xp
- **xp**: Help window fits its key table, every row reachable
- **xp**: own start mark and user picture, no Windows flag
- **xp**: no false launch balloons for launchers and live programs
- **hud**: the speaking figure hops without scaling, no resampling
- **draw**: 14 px chrome fonts only where chora-xp asks for them
- **xp**: a minimized task button never takes the pressed look
- **xp**: a press with Super held is not a tap of Super
- **dist**: install the whole tiling flavor in the RPM
- **dist**: install the whole tiling flavor in the Arch package
- **tooling**: review fixes of merge-gate and ci-local
- **mc**: place every panel at 1280 and send the slot after a success
- **assistant**: answer a click on a card that only followed its view
- **assistant**: leave the plain field at scale 1, it cannot paint 2
- **mc**: keep the arrows of the key pair together
- **assistant**: show Stop only while the helper runs, muted
- **assistant**: scroll the transcript by whole messages
- **assistant**: draw the plain field at the scale, its hint muted
- **mc**: put the hover mark beside Esc Close and Show
- **assistant**: drop the question hint when the card is answered
- **assistant**: reopen the chat of the only helper after Esc
- **mc**: draw the state dot in the colours of the theme
- **assistant**: keep the card above the hint when the field grows
- **mc**: give each empty panel of the morning its own row
- **assistant**: draw into the slot of a look, with its page, fonts
- **mc**: show a helper whose state carries the fields of the host
- **mc**: 14 px head, serif fallback, calm notes, empty pages
- **mc**: order the full key list by task, keep arrows off the plus
- **tiling**: traps before pids, session-safe logout, pictures dir
- **tiling**: ask the system off the loop, exact gaps, Super swallow
- **tiling**: align tooltips, spell keys alike, mark today
- **tiling**: a theme reload restarts bar and wallpaper, no units
- **tiling**: push tag and layout changes to the bar with a signal
- **tiling**: visible launcher selection and a prompt on both menus
- **tiling**: balanced key list margins, keycap text inside its cap
- **tiling**: even bar spacing, left-aligned layout word, plain Keys
- **tiling**: find power and screenshot scripts, add Screenshot entry
- **tiling**: keep the key list clear of the bar
- **tiling**: Ink wallpaper, drawn icons and a bar that reads
- **tiling**: key list, chip, terminal and launcher in the palette
- **tiling**: count the gaps from the outside of the border
- **gate**: signal the programs of gate-lib-selftest once they run
- **gate**: continue stopped programs when a gate ends
- **gate**: wait for the walk to take the keys in construct key cells
- **construct**: count one window as 1 window
- **kit**: quote words as each language does, and no jargon in German
- **construct**: name the side of a window in the list right
- charge the watchdog for a backdrop by its area
- capture the frame behind a backdrop as a framebuffer effect
- blur a backdrop again only when the frame behind it changed
- **chat**: keep the card of the balloon above its hint
- **chat**: set the placeholder of the field after the caret
- **chat**: size the balloon of the figure to its content
- **chat**: name the helpers of the list as voice writes them
- **hud**: give the sentence the look of a balloon chat
- **assistant**: show a sentence in the open chat in every look
- **assistant**: size the pictures of the card at every scale
- **draw**: draw a figure where it stands in its sheet
- **gate**: wait for the keyboard of the xp window before typing
- **construct**: check the folder chat against the glass tokens
- **assistant**: give the card its logical size at every scale
- **assistant**: name the words of the island for readers
- **assistant**: draw the card of a question that names its helper
- **construct**: name the voice state beside the ring of the HUD
- **hud**: name the theme when the host listens
- **xp**: place the paperclip from the work area above the taskbar
- **xp**: reserve the taskbar in the work area of the core
- **hud**: give the mc and construct folders their dark colours
- **assistant**: read the place again until a screen is claimed
- **assistant**: draw no rows while the chat is open
- **xp**: say Sound unavailable instead of wpctl not found
- **tiling**: show only words of a person on the panel
- **assistant**: draw no row for the helper of the open chat
- **construct**: put the HUD rows at the top left
- **xp**: open the chat from the paperclip and drop the Chat pill
- **assistant**: show each look in its own HUD folder by default
- **chat**: draw no transcript while the list shows
- **chat**: open the list with an empty field
- **construct**: take a press beside the cube for the pane under it
- **construct**: put a pulled pane back with the same plain drag
- **construct**: give the pane ahead and fly into a new pane
- **construct**: list every window by name in the window list
- **construct**: hide OPEN CUBE in a pane and keep it on screen
- **construct**: list the cube keys in the help only with a cube
- **construct**: draw notices in one line under every voice_hud
- **construct**: the review of the pull from the cube
- **construct**: float no window off a face at a grazing angle
- **construct**: keep the cube chips clear of the desktop icons
- **construct**: hide the cube chips while the camera is in a pane
- **construct**: give the keyboard to a window as the fly-in starts
- **construct**: end the xp of the cube with the construct
- **construct**: draw the focus glow without time
- **construct**: put a pulled window in front of the cube face
- **core**: click answers in the test mode without the injected mark
- **chat**: keep the field and its keyboard when a helper is picked
- **core**: count injected key presses as acts in the test mode
- **assistant**: name the search path when --use-hud would miss it
- **voice**: say that a key question needs a key press or a click
- **voice**: say helper and Sie, spell keys once, title the question
- **assistant**: show Close in the helper list as in a chat
- **chat**: ask to pick a helper first, and name helpers in the list
- **assistant**: let the voice tools show and close the chat
- **xp**: leave the keyboard alone only without an opened window
- **construct**: end the window manager of a pane with the construct
- **construct**: reject an empty command of a window manager
- **construct**: drop a manager pane whose manager is unknown
- **core**: pass a toplevel under a pane only for a focused node
- **core**: keep the texture for a damage-less attach, import lazily
- **core**: put the damage of each commit into the texture at once
- **demo**: the cube scene takes the renamed test flag
- **construct**: a step of the cube takes the animation clock
- **xp**: move the desktop buttons when the speaker grows
- **construct**: say the walk notice in the person's language, rarely
- **construct**: warn when a start or close key has no modifier
- **tiling**: make the panel honest and let it follow the outputs
- **volume**: run wpctl off the main loop and name every state
- **xp**: list only the keys that xp holds in Keyboard Shortcuts
- **kit**: log each refused grab once and tell which keys are held
- **core**: allow listed media keys only for window managers
- **core**: let a key choose its manager only where the person is
- **core**: pass unbound Super keys to programs again
- **construct**: start nothing from text typed into the space
- **assistant**: keep the HUD on its screen while the focus moves
- **tiling**: show the panel only on the outputs of tiling
- **core**: open the window of a key where that key's manager is
- **core**: keep unbound Super keys out of programs
- **core**: let a window manager grab media keys without a modifier
- **kit**: read a method call without params as its defaults
- **assistant**: clip the slot box and refuse folders that read badly
- **assistant**: hide the helper rows with the panel that holds them
- **core**: a still virtual output leaves the winit window still
- **construct**: cube backlog after the review
- **construct**: a floating window never shows nowhere
- act on the review of the demo branch
- **construct**: cube modes after the review
- **kit**: received springs sit at the window that takes them
- the review of xp on virtual outputs
- **xp**: give a dragged window only past a margin of its face
- **construct**: end a pulled drag inside a window too
- **core**: unpin a press only for the owner of its pane
- **core**: let a given drag leave the pane that it started in
- **core**: the review of the windows of a removed output
- **construct**: act on the review of the cube
- **core**: the review of the input through a pane
- **construct**: act on the review of K5
- **core**: keep the perspective and empty boxes behind the eye
- **core**: give a plane behind the eye an empty box
- **core**: tie a give during a drag to its press and its window
- **core**: the review of the virtual outputs
- **kit**: keep the 3D wobble finite and take a view of x and y
- bound the HUD host state ask and retry only a missing call
- the HUD host asks voice for its state after voice connects
- **voice**: key claims by connection, ask a key for security
- **core**: drop the indicator.claim grant
- **core**: keep a live name for its program
- only named HUDs claim the indicator and answer
- **core**: fit layer textures to the GL texture limit
- hand choractl run the token of a plugin in its environment
- **core**: keep layer ids, pass results and steps per output
- **core**: block input only on what the budget left dark
- **core**: stamp no token for a confined call, bind acts to conns
- **core**: bind the act of a token to the client that received it
- **core**: block input on left-out layers in one place, per output
- **core**: draw a plane wider than 4096 pixels whole
- **core**: never repeat a layer texture version after an eviction
- log a poisoned display lock and clean a scratch dir first
- keep one display open and give each test its own directory
- **mc**: align a scene with one query for its texts
- **core**: lay out only the nodes that a query matched
- **mc**: clear the effects on a drop and restore the blur
- **hang**: abort after a ceiling of wall time without a beat
- **core**: keep the last words of a stack overflow and a bad heap
- **core**: call the host busy only for frames that mostly waited
- **mc**: remove the write effects of one show in one transaction
- **mc**: end the write effect and stay plain after a drop
- **core**: count no wait for a CPU as the cost of a frame
- **core**: leave last words at every end of the core
- **hang**: count no stall while the watchdog thread did not run
- **core**: restore the focus and keep the pin after a popup ends
- **core**: make the plane press a pointer grab of Smithay
- **assistant**: act on the findings of the review of H7
- **assistant**: keep the Chat row of a flavor off the title bars
- keep the give keys while a plugin grabs its other bindings
- one key table, regrab on release, a bounded forward of a key
- **effects**: end wobbles and lamps in the driver, as reviewed
- **core**: keep image steps inside slice cells and count their cost
- **core**: sample image nodes from halvings and snap to pixels
- **chat**: run the field on a pipe and keep held text
- **chat**: tell when the field is shown and wait for it in the test
- **core**: bound the layer caches and step only on screen
- **core**: keep both placements of a layer context order
- **assistant**: reload a folder only when a rule names another
- **assistant**: read the context at the start and calm the switch
- **assistant**: keep the card and the balloon in the work area
- **assistant**: refuse a question whose detail does not fit
- **kit**: clean the card text and accept both kinds of question id
- **mc**: keep the clock duration within the core limit
- **core**: limit drag and drop to the real press of its client
- **kit**: harden the chat model against hidden text and gaps
- **core**: bound the split and hit nested contexts sorted
- **core**: block input on left-out layers of a context
- **kit**: open a HUD file without a link and check its inode
- **draw**: bound and confine the fonts directory of a HUD folder
- **draw**: bound the cost of an SVG of a HUD folder
- **core**: take only a command as the act of a person
- **core**: stamp activation tokens with the act of a person
- **tiling**: leave the focus of a handed window to its receiver
- **core**: block left-out planes and map planes at the near plane
- **hud**: bound values, harden the loader, check panels
- **words**: helper states, singular notice, no jargon
- **ask**: fix the place of the card on top, drop text_of
- **voice**: harden the chat transcript and cancel
- **voice**: reject an ACP request too long to show whole
- **voice**: show the whole ACP request and name the keys
- **voice**: ask ACP permissions that run or write by key
- **construct**: take over handed-over windows without focus loss
- **kit**: give the notification calls a timeout of 2 seconds
- **xp**: refuse only a plain command of a missing program
- **xp**: drop the message for a program that ended without a window
- **xp**: say when a program starts, fails or opens no window
- **voice**: refuse confirm, talk and mute from a confined caller
- **hud**: ignore an injected press on an answer button
- **voice**: refuse indicator.claim and confirm.answer when confined
- **test**: end the descendants of xp.sh in its cleanup
- **notify**: end without an error when the bus name is taken
- **assistant**: hang the bubble below the figure at a top corner
- **hooks**: skip the length of amend! subjects
- **hooks**: say session files belong outside the repository
- **hooks**: split the staged file on newlines only
- **hooks**: allow only real Rust comment forms
- **hooks**: count // after a colon outside a string as a comment
- **hooks**: start a heredoc only when its delimiter follows
- **hooks**: pair renames in no-inline-comments
- **hooks**: let commit-message-length handle git's own text
- **hooks**: make no-inline-comments scan strings and scripts
- **ipc**: compare effective confinement of a name's connections
- **scene**: end a camera move exactly on its target
- **scene**: refuse the inverse of a nearly singular matrix
- **scene**: animate a rotation as a turn, not a matrix blend
- **persona**: check the tools of the companions before the start
- **persona**: stop the companions by unit when choractl run has one
- **persona**: delete a stale last_look.json at session start
- **persona**: refuse a PERSONA_KEEP that is not empty
- **persona**: let act.py type take the rest of the line as text
- **persona**: remove only a session directory that the script made
- **persona**: click in the pixels of the last capture
- **construct**: a launcher under a forking shell keeps its window
- **voice**: ask before load_view
- **choractl**: confine the event connection of mcp
- **core**: apply the client socket scene rules when confined
- **core**: mark injected keys in input.key events
- **voice**: ask before every tool that is privileged
- **voice**: say the whole call in spoken confirm questions

### Refactor

- **voice**: build the HUD question in question_text
- **kit**: show settings.get through Plugin::shown_settings
- **xp**: drop the stale expect allow of Wm::grab
- tell a handler of node.key whether the key was injected
- **draw**: mark a file layer whose path a sender named
- **construct**: name the row that a press on the list chooses
- build the log subscriber in one function for init, tests
- **kit**: find the shared entries with one helper
- **shared**: move the power and screenshot scripts
- **core**: compose the cursor and the output in render.rs
- **mc**: move the head hit and its alignment out of long functions
- **mc**: dispatch client events and presses on their own
- **core**: name the test flag after what it does
- **core**: keep the dispatch functions within their length
- **construct**: split main.rs into files by function
- drop a stale allow and a second output lookup
- name the module offscreen, not virtual
- render outputs into textures in virtual.rs
- **kit**: check a when on its own and name its keys
- **xp**: remove the unused position of a taskbar
- **xp**: drop the zbus dependency of the moved server
- **hooks**: replace the session files script by a fail hook
- **scene**: make Transform::compose crate private
- **render**: tie element counters to Variant at compile time

## v0.9.0-rc.3 (2026-11-01)

### BREAKING CHANGE

- security settings no longer take a spoken yes.
- voice restart is gone; call voice settings.reload.
- the setting allow is gone; use allow_programs and
allow_methods.
- the default of modifier is Super, not Ctrl.

### Feat

- **mc**: tag the root group for panes, so a pane shows the regions
- **construct**: give the keyboard to a plugin while flown into its pane
- **chora**: name the nested window after its Wayland socket
- **voice**: raise history_tokens to 24000 and document the notes
- **voice**: keep a note of dropped turns and only the latest picture
- **voice**: tell the agent that construct ignores the scene camera
- **voice**: trim large tool results before they enter the history
- **voice**: tell the agent to batch commands and tools per round
- **voice**: claim a visible result only after the screen confirms it
- **voice**: add listen: always | push_to_talk
- **construct**: anti-alias the edges of panes and mirrors
- **xp**: answer a question by the balloon's buttons or show its keys
- **mc**: answer a confirmation by a click or show its keys
- **construct**: answer a question by a button or show its keys
- **construct**: show the key bindings in a help panel
- **voice**: voice.confirm has buttons while a claimer is connected
- **voice**: answer a question by the button of a HUD
- **core**: the press of a move names the window that asked for it
- **voice**: confirm security settings by a key press only
- **voice**: refuse inject_input while a key question waits
- **winit**: name the window after the startup program
- **construct**: mark the focused pane
- **construct**: place new panes only in the view
- **construct**: show the voice HUD, its questions and the pane label
- **construct**: read the style section
- **voice**: bound the history with history_tokens
- **voice**: start a nested chora for a window manager in a pane
- **construct**: a smooth glossy floor with a crisp horizon
- **construct**: Super moves and resizes panes, as in tiling and xp
- **construct**: hand a dropped pane to the window manager or plugin under it
- **tiling**: drags move floating windows, also from a title bar
- **core**: the interactive move of a client becomes a drag of its window manager
- **core**: window managers with claims share a pointer binding
- **construct**: a window list, windows of other clients on request, and window.give
- **choractl**: give a window to another window manager
- **mc**: adopt given windows, give them away, and --output
- **tiling**: window.give and a drag out to another window manager
- **xp**: hand a dropped window to the claim under the drop point
- **kit**: window.give and the window manager under a point
- **voice**: an allow list of programs and methods that never ask
- **voice**: a yes with always allows the action for the session
- **voice**: emit each question with its full command
- **voice**: place the indicator and let plugins claim it
- **mc, xp**: claim the state indicator of the voice agent
- **mc, xp**: show the spoken confirmation of voice.confirm
- **construct**: keys for a terminal, a launcher and a close
- **construct**: ground, horizon, dust, shadows and mirror images
- **construct**: construct files, overlay mode and plugin panes (K2a)
- **mc**: tag the page of each scene as meta:pane=NAME
- **mc**: notify a broken settings file at the start
- **xp**: a paperclip assistant shows the voice agent
- **mc**: an overview scene of all scenes as N | Title
- **mc**: scene.rename and scene.move
- **mc**: order the scenes by the setting order, number them by place
- **construct**: add the construct flavor target and service
- **construct**: walk on a leash and fly into program panes
- **mc**: give the agents as many columns as there are agents
- **mc**: scroll a long page with the wheel
- **mc**: wrap panels into new rows on a narrow output
- **voice**: take yeah, yep, jo, klar and gerne as yes
- **voice**: tell the LLM to read files with read, as data only
- **voice**: read project files and folders with the read tool
- **voice**: say when the agent is ready
- **voice**: name the arguments in the confirmation questions
- **voice**: dump the LLM requests and raw streams on demand
- **mc**: set the number of the scene head faint in Pagella
- **mc**: let the app of a scene name an app content
- **mc**: set the scene head as "3 | Agenda" in the text column
- **voice**: remove the agent's own nodes without a question
- **voice**: acknowledge tool rounds and finish announced actions
- **voice**: speech during a reply reaches it at once
- **voice**: the agent changes its own settings after a spoken yes
- **mc**: place the built-in scenes on the shared grid
- **mc**: one grid and page for all scenes in the settings
- **mc**: hang the scene number in the margin beside the title
- **mc**: trace an inner hairline in write, fill fades in behind

### Fix

- **construct**: adopt a window handed over to a pane that only mirrors it
- **mc**: refuse pane.focus for a scene that is not the current one
- **voice**: send no Cancel for a question that a muted speaker never queued
- **voice**: put the note of dropped turns in the first system message
- **voice**: refuse a confirm detail over the cap, show it whole in the HUD
- **voice**: hide header values before cutting a tool answer
- **construct**: a window hidden for lack of room becomes an own pane
- **mc**: claim windows only with --manage-windows
- **construct**: attach a window that finds no room, hidden
- **construct**: sight hides a pane as the frame does
- **mc**: claim the work area, not the whole output
- **construct**: cap the confirm detail on screen
- **construct**: act only on a pane that shows
- **voice**: keep call_method arguments for the own settings.set
- **core**: tell a client why the watchdog disconnects it
- **core**: hand every free window to a definite manager or show it
- **voice**: unwrap patch in call_method settings.set
- **voice**: an older utterance does not keep a question open
- **voice**: block only a retry with the same values
- **voice**: ask by key when a reload changes a security key
- **voice**: match allow_programs against the whole argv
- **voice**: stop the speech of a question when it is answered
- **voice**: hide secrets in the answers of settings.set and in the journal
- **voice**: restart the tool process when chora_mcp changes
- **chora-xp**: find the theme of a source tree
- **construct**: hide a pane with a corner behind the eye
- **xp**: refuse to start without an absolute XDG_RUNTIME_DIR
- **xp**: follow only the voice agent, and show it asleep
- **mc**: let only the voice agent set the voice state
- **mc**: accept a string or a number as the voice.confirm id
- **construct,mc**: write the shaders only to the runtime directory
- **construct**: fit every text of the HUD into its panel
- **construct**: the panes now turn with you when you walk
- **construct**: keep the given windows over an overlay load
- **tiling**: grab again when the claim appears
- **tiling**: keep floating windows inside the work area of their output
- **tiling**: test a drop against the own outputs, not the claim box
- **core**: share button grabs only between clients of the main socket
- **tiling**: drag the window that asked for the move
- **core**: a move needs the press that still holds the pointer on the window
- window.give answers the surface id in every window manager
- **kit**: export window.give as privileged
- **mc**: claim the output, so that window managers can give mc windows
- **core**: only window managers receive windows, and reservations expire
- **voice**: emit the HUD events only as the client voice
- **voice**: refuse allow and confirm changes by voice
- **voice**: hide secrets in voice.confirm and settings.get
- **voice**: stop a tool call that fails twice the same way
- **voice**: make mute, talk and indicator.claim privileged
- **voice**: drop the unprivileged restart method
- **voice**: ask before the region methods and window.give
- **voice**: end the always words of the session on new settings
- **voice**: narrow always and split allow into programs and methods
- **voice**: refuse unknown keys in the indicator place
- **voice**: send the voice.confirm id as a text
- **voice**: keep a confirmation while a yes in time is heard
- **construct**: say that pane.move takes an offset, and answer the place
- **render**: make node textures opaque where a window is opaque
- **construct**: move the focus to the pane flown into
- **construct**: mark the open window list for the gates
- **construct**: keep a walk during a drag
- **voice**: never speak a cut fragment; retry with a note, then brake
- **tiling**: a window from another window manager keeps its size
- **voice**: refuse a broken settings file on restart too
- **voice**: end no sentence at a colon
- **voice**: keep secrets out of the journal
- **voice**: stop on a broken settings file
- **voice**: read a file through one checked handle
- **voice**: never cut a security value in a question
- **voice**: ask before the privileged methods of the plugins
- **voice**: nudge only after an explicit announcement
- **voice**: only a pure yes or no after the question answers it
- **construct**: the program of fuzzel with its token becomes a pane
- **kit**: construct and mc scene files save through a new 0600 temp file
- **construct**: expire program panes that wait, and never save them
- **construct**: a radius of settings.set applies without a construct file
- **construct**: a click inside the flown pane keeps the way back
- **construct**: pane.remove keeps a window that refuses to close
- **xp**: the balloon says when the mic does not switch
- **mc**: scene.rename refuses a file that exists
- **mc**: a color that a theme lacks comes from the default theme
- **mc**: a new window takes the keyboard, except one of app.open
- **voice**: run one more round after an answer that stops mid-sentence
- **voice**: keep the user's level in the echo canceller
- **voice**: measure command words by their speech, not their length
- **voice**: normalise each utterance before the ASR
- **kit**: save only the settings that differ from the defaults
- **kit**: start with the defaults when the settings file is broken
- **mc**: only the user moves the keyboard
- **mc**: app scenes get their head at once and go with their app
- **mc**: one box for the window of an app scene
- **mc**: remove the probe of a region update right after measuring
- **mc**: show the sides on a narrow output
- **voice**: new words in the last tool round get rounds of their own
- **voice**: catch any first-person announcement without a call
- **mc**: build the page anew when a panel moves, fills or empties
- **mc**: show whole lines in panels and let a long title wrap
- **mc**: set the bar of the scene head as a hairline on the baseline
- **voice**: catch announced actions without a tool name
- **voice**: keep the note of a stopped answer out of the speech
- **mc**: change the agent panels in place, without a write
- **core**: warn once when no font has the family of a text node
- **voice**: tell the LLM to speak to the user, not narrate
- **voice**: give the LLM a lean schema for its own settings.set
- **voice**: drop a transcript that is the ASR prompt
- **mc**: put the window of an app scene in its text block
- **mc**: give foot in the drawer all colors of the theme
- **mc**: use lining figures for the number of the scene head
- **voice**: prompt rules for short answers and safe nodes
- **voice**: hear a short yes right after a spoken question
- **voice**: time a confirmation from the end of its question, 15 s
- **mc**: reject a cell outside the grid of the settings
- **mc**: change region texts in place, without a cross-fade
- **mc**: grow the running head with scale.base
- **voice**: adapt the barge-in floor to the user's level
- **voice**: lift quiet mic speech for the VAD
- **voice**: tell the LLM why a spoken confirmation failed
- **voice**: count a confirmation answer by its yes or no word

### Refactor

- **mc**: keep build, show and method under the line limit
- one claim-under-point helper and one MOVE constant in chora-scene
- **voice**: drop the old note and fix the prompt on own nodes
- **kit**: one launch for mc and construct
- **mc**: a fixed 4x3 grid per scene, one page for all scenes

## v0.9.0-rc.2 (2026-10-31)

### BREAKING CHANGE

- model is llm_model, language is asr_language.

### Feat

- **voice**: TLS for https and wss with rustls, ring, system roots
- **voice**: per-server settings, languages and the ASR prompt
- **voice**: ask before agent_start
- **voice**: method agent runs the agent tools without the LLM
- **mc**: plan, question, transcript and focus of sub-agents
- **voice**: ACP sub-agents, one per project, by name and voice
- **voice**: run the coder in sandbox_cmd unless unsandboxed

### Fix

- **voice**: wait for the process groups of agents at the exit
- **voice**: time a question of an agent from when it is spoken
- **voice**: cut the words of an agent at 16 KiB per message
- **voice**: bound the texts that wait, replace old questions
- **voice**: ask "Immer erlauben?" before a yes allows always
- **voice**: resend the last refused state after the rate window
- **voice**: the LLM calls no method of the agent's own clients
- **voice**: voice.confirm is privileged
- **choractl**: confine the view connection of the MCP tools
- **voice**: read agent lines in a task, safe against select
- **voice**: make tests/fake-acp.sh executable
- **voice**: ruff docstring findings in corpus.py
- **voice**: the language rule closes the prompt, empty asr_prompt
- **voice**: find the project of agent_start in any case
- **voice**: a name word differs by one edit only from 5 letters
- **voice**: mark the spoken answer, ask "NAME möchte ..."
- **voice**: speak a short question: file name and verb

### Refactor

- **voice**: drop expect outside the agent and client files
- **choractl**: allow CLI prints at the crate level
- **mc**: drop expect on agent push and shader path
- **xp**: log a binding that does not parse
- **tiling**: drop expect on window lookups and index math
- **kit**: return an error for an unparsable grab
- **scene**: document the expect invariants
- **core**: handle or document unwrap and expect

## v0.9.0-rc.1 (2026-10-30)

### Feat

- **voice**: ask before each run_program, with the program name
- **ipc**: hello confined gives a connection the client-socket rules
- **voice**: emit voice.state and voice.say instead of mc calls
- **mc**: follow the voice.state and voice.say events
- **core**: let clients emit events under their own name
- **choractl**: view load prints the input settings of the core
- **input**: null in input.configure resets a setting
- **voice**: the indicator dot and health probes of ASR and TTS
- **voice**: voice.health, server backoffs and spoken errors
- **voice**: coding tasks with the working state and spoken results
- **voice**: the TTS language follows the language
- **choractl**: a view sets input devices with input:
- **core**: input.configure sets key repeat and libinput
- **scene**: input settings for keyboard, touchpad and mouse
- **core**: watch a heartbeat of the main loop in a thread
- **session**: screen sharing through xdg-desktop-portal-wlr
- **tiling**: a tags module in the waybar of the flavor
- **tiling**: get_tags answers the tags of each output
- **tiling**: MOD+Shift+Escape locks the session
- **tiling**: add panel, wallpaper, idle lock and outputs
- **session**: start chora as a uwsm session

### Fix

- **choractl**: mcp confines its connection to the main socket
- **flavor**: read the layout for waybar with jq
- **session**: keep the portal while another session is open
- **hang**: write the watchdog lines with a raw write(2)
- **core**: quit on a failed start only if no client said hello
- **tty**: find the node of a removed GPU among the open devices
- **core**: a crashed window manager with windows keeps the core
- **session**: restart a portal of an earlier session
- **probe**: destroy each buffer when the core releases it
- **core**: quit when the startup command fails alone (#57)
- **render**: throttle frame callbacks of hidden windows (#60)
- **core**: stop a shader read at the size limit (#31)
- **mc**: switch regions, docks and themes without flicker
- **mc**: agenda.sh no longer hangs on an org-agenda prompt

## v0.8.0-rc.1 (2026-10-30)

### Feat

- **voice**: levels subcommand for the mic and echo check
- **voice**: state machine with barge-in, queue and mc reports
- **voice**: note of a stopped turn in the history
- **voice**: local word matcher with fuzzy phrases
- **voice**: barge-in level from the played sound
- **core**: privileged methods need methods.privileged
- **mc**: tests collector for the test overlay
- **mc**: system collector for Mission Control
- **mc**: morning collector for the morning paper
- **mc**: morning paper and Mission Control as built-in scenes
- **mc**: agents scene, status line, notify.push and region.solo
- **mc**: scene pages, panel rules, opacity and blur
- **mc**: agenda collector fills the next entry for the footer
- **mc**: regions around the page with rules, toggles and presets
- **voice**: spoken replies, the confirm method and ask
- **voice**: tool loop with history, token budget and prompt
- **voice**: sentence splitter and speech with a playback buffer
- **voice**: MCP tools with reconnect, timeouts and confirm rules
- **voice**: LLM client with SSE, tool calls and a fake server
- **xp**: apply a requested look to the window group (D58)
- **tiling**: apply a requested look of a window (D58)
- **voice**: transcribe each utterance, interim text
- **voice**: ASR and TTS clients with fake servers
- **voice**: replay harness with labels and metrics
- **mc**: agenda collector leaves the date to the running head
- **mc**: book typography with styles, chapter heads and margins
- **core**: line height, weight, style, tracking, features of text
- **kit**: settings.reload applies hand edits in every plugin
- **voice**: core loop, chora client, methods and PTT grab
- **voice**: mic and speaker commands, earcons, recordings
- **voice**: speech gate with onset, pre-roll limit and PTT
- **voice**: settings with a default for each key (D62)
- **voice**: Silero VAD v6 on tract, 24 to 16 kHz step
- **xp**: Log Off ends chora with choractl quit (D63)
- **kit**: settings.set merges a patch, a file takes defaults (D62)
- **core**: session.quit ends chora, choractl quit calls it
- **mc**: agenda collector puts the time in a column
- **mc**: write effect for new text, hud.say and two columns
- **mc**: Inter as the default font
- **mc**: agenda collector shows only what is due today
- **scene**: animate the numeric uniforms of effects
- **xp**: settings in chora-xp.yaml in place of the flags (D62, D63)
- **kit**: a public dispatch for plugins with a loop of their own
- **mc**: collector for the org agenda of today
- **mc**: experimental full-screen views with panels and a HUD
- **tiling**: settings and a default style for new windows (#99)
- **kit**: plugin settings with settings.get and settings.set
- **xp**: open folders with the file manager of --file-manager
- **mcp**: apply_commands tells of requests to the owner
- node.requested action update for the look of a node (D58)
- warnings with stable codes in the answer of scene.apply

### Fix

- **voice**: barge-in between sentences and with its first words
- **mc**: wait anew for ended apps after each launch and build
- **xp**: keep the start menu closed under a session dialog
- **xp**: cap the clock format at 64 characters
- **tiling**: compile style shaders at settings time
- **tiling**: refuse a run binding without a command
- **kit**: refuse unknown keys and apply old settings again
- **core**: answer an unknown key of input.grab with -32602
- **mc**: keep the HUD line to its last 160 characters
- **scene**: limit the tree to 64 levels below a root
- **mc**: keep windows and outputs, leave scenes of ended apps
- **mc**: clamp panel sizes on a small output
- **probe**: destroy the shm pool after each paint
- **mc**: leave out a scene that the core refuses, keep the rest
- **mc**: bound the scene grid to 24 and check cells without overflow
- **mc**: mark the methods that start programs privileged
- **kit**: cut error messages of methods to 512 bytes
- **plugins**: refuse devices and /proc files in settings
- **kit**: read user-named files capped and regular only
- **mc**: regions touch only nodes that exist, smaller functions
- **core**: layer_over_budget also checks the layer ancestors
- **scene**: a mixed look request names the owner-only props
- **core**: refuse a look request that its owner would not get
- **choractl**: MCP tools of dotted methods reach the plugin
- **kit**: take settings back when the plugin cannot apply them
- **kit**: save settings in a new 0600 file, through a symlink
- **core**: settings.set and settings.reload need settings.write
- **tiling**: drop MOD+p, MOD+r runs the launcher
- **xp**: log a session command that fails
- **xp**: the Recycle Bin makes a missing trash or opens the home
- **scene**: refuse an empty update of a node of another client
- **mc**: keep the write effect at one path in the runtime dir
- **mc**: subscribe after the kept scene goes, adopt a surface once
- **xp**: the Terminal of the start menu runs the terminal setting
- **kit**: apply settings before the save, keep them when it fails
- **kit**: a settings patch replaces a map entry that is no struct
- **kit**: a settings file without an Option key takes its default
- **xp**: give Date and Time the calendar icon of the theme
- **xp**: log a start menu command that is not on PATH
- **mc**: agenda collector says why emacs did not run
- **mc**: give a second app.open of a program its own scene
- **xp**: say that the old flags live in chora-xp.yaml
- **core**: log the exit status of the startup command
- **tiling**: refuse a style effect whose shader file is missing
- **xp**: announce after the scene, so kept windows keep their place
- **choractl**: pass the activation token to GTK 3 programs
- **mcp**: call_method shows a layout that exists (#56)
- **mcp**: a failed subscribe_events keeps the old subscription (#53)
- **mcp**: describe_screen hides windows of hidden groups (#54)

### Refactor

- **voice**: remove the Python agent and its unit
- **kit**: settings.set private, one settings() per call
- **tiling**: always send the style when taking a window
- **xp**: no Default impls that repeat xp.yaml
- **xp**: the start menu reads the settings, no copy
- **xp**: rebuild wallpaper, taskbar and menu on each change
- **xp**: one wallpaper type, resolve the default where used
- **mc**: rename views to scenes
- **xp**: move onto the plugin runtime of chora-kit
- **xp**: remove the Explorer, folder windows and file dialogs
- **tiling**: use the plugin runtime of chora-kit
- move the plugin runtime into chora-kit

## v0.7.0 (2026-10-30)

### Fix

- **voice**: a coding task ends before its result waits to be spoken
- **voice**: a null task or an odd JSON line does not stop the agent
- **voice**: a coding result is spoken only, code_status repeats it

### Refactor

- **voice**: the coder derives its tool names and next clone

## v0.7.0-rc.3 (2026-10-30)

### Feat

- **voice**: coding tasks in clones of the inventory, with status and cancel
- **voice**: an LLM that reads images gets the screenshot of describe_screen
- **voice**: the conversation goes on at the next start
- **voice**: the prompt takes only the sections of the docs that the LLM uses
- **voice**: the LLM reads the system prompt at the start
- **voice**: turns keep soft word ends and drop noises
- **voice**: CHORA_MCP runs the chora tools in another place
- **voice**: a system prompt that explains chora, with its docs after it
- **voice**: the level of speech follows the quiet room
- **voice**: PREROLL_MS sets the mic time a turn keeps
- **voice**: PipeWire echo cancellation for the agent's mic
- **voice**: stream the answer, one mic, and a barge-in without a gap
- **voice**: rebuild the agent with chora tools and push-to-talk
- **mcp**: a run_program tool for agents
- **ipc**: client sockets need methods.call and activation.token

### Fix

- **voice**: coder reports are tool results, the history file keeps requests
- **voice**: a coding result is in the history before the next answer
- **voice**: coding tasks stay in the inventory and in their clone
- **voice**: the prompt names the limits of the scene that the tests showed
- **voice**: a failed turn keeps the turn before, echo peaks do not add up
- **voice**: the history file keeps the words of all sessions only
- **voice**: the image content of mcp 2.3 has mime_type
- **voice**: the earlier session is a note in the system prompt
- **voice**: silent work needs a sentence to stop, the LLM keeps its last shaders
- **voice**: a new session takes up the words only, no stale tool results
- **voice**: a barge-in without words hands the turn back to the LLM
- **voice**: soft speech counts for the noise filter, 200 ms by default
- **voice**: a barge-in counts through damped dips of the voice
- **voice**: the warm-up has a user message, the prompt explains transactions
- **voice**: a refused LLM request logs the answer of the server
- **voice**: earlier turns keep short tool texts, the log shows why a reply ended
- **voice**: the prompt forbids taking windows from the window manager
- run answers with the error of systemd-run
- **voice**: keep the last 20 turns with their tool calls
- **voice**: allow 10 rounds of tool calls per turn
- **voice**: the settings of the agent live in ~/.config/chora/voice.env
- **voice**: log a reply that the end of the context cut
- **voice**: AddDevice is the quadlet key for the GPU devices
- the emergency reset drops the waiting activation tokens
- **voice**: pin the runtime image of the models by its digest
- a symlink to the socket names the unit of the socket
- **voice**: a new environment takes no release after a set date
- a view name that ends in .yaml loads no file by path
- surface.created carries a token that follows the first commit
- **voice**: a key turn starts at the press
- **voice**: ask the LLM not to think
- **voice**: the LLM gets no put_asset
- **voice**: flush the mic before a reply again
- **voice**: keep words said during the ASR, ASR pieces of a second
- **voice**: answer a request held for more words, also when none follow
- **voice**: an empty reply withdraws its request
- **voice**: a key held during the ASR goes on, a noise withdraws
- **voice**: a key stop withdraws a request without an answer
- **voice**: end a turn on a failed transcription, log the coder output
- **voice**: a held key does not keep the echo of a delegate result
- **voice**: a turn ends after 30 s, a held key keeps its frames
- **voice**: no start limit, start the agent also with remote models
- **voice**: one coding task at a time, the first comma after 40 characters
- **voice**: start the models only when the agent starts
- **voice**: start the agent only in a chora session
- **voice**: stop aplay on a late barge-in, keep a cut tool call
- **voice**: exit when arecord ends, keep the quiet start of a turn
- **voice**: words before tools without a period, key events without a filter
- **voice**: recover from a refused history, speak words before tools
- **voice**: log LLM error events, a plain read loop in talk()
- **voice**: answer after a failed turn, keep calls that ran
- **voice**: no two user turns in a row, push-to-talk waits for chora
- **voice**: fix the history, echo and push-to-talk bugs from review
- **voice**: end aplay after the sound, so the agent listens again
- **voice**: German ASR, whole replies, a log line per turn
- **voice**: listen again after a reply, and let the user barge in
- **voice**: wait for quiet before a turn may start
- **voice**: turns survive failures, push-to-talk ends cleanly
- **voice**: quadlets that pass the model and harden the containers
- **run**: the fallback sends stderr to the null device again
- **view**: wait for a plugin whose unit runs already
- **xp**: suspend without blocking the window manager
- **view**: a stable hash in the name of a view unit
- **view**: a view unit per core, the fallback keeps stderr
- **run**: the fallback keeps the program in the process group of the caller
- **choractl**: views run exec as written, restart stays in its session
- **view**: one plugin unit per session
- **safety**: the revoke binding keeps windows that got no protocol
- **mcp**: run_program needs activation.token on every socket
- **restart**: restart only a running plugin
- **ipc**: a client socket clears the focus only after a press
- **choractl**: fall back without a running manager, sync only main grants
- **run**: fail instead of starting twice, reap detached programs
- **ipc**: a client socket focuses only after a press, revoke ends tokens
- **grants**: refuse the capabilities of client sockets
- **plugins**: start apps with choractl run
- **view**: start plugins as units that choractl restart finds
- **run**: literal arguments, a unit that outlives a launcher

### Refactor

- **voice**: no start timeout, the setup pulls the image
- **voice**: counters in ms, a cut call in run_calls
- **voice**: levels shows the thresholds, one rms per frame
- **voice**: a full TTS URL, a clearer barge-in log and key loop
- **choractl**: one systemctl helper, views list only started plugins
- one press rule arm, all clients in the revoke texts
- **grants**: one revoke_all for the binding and the reset
- the revoke binding takes all grants, gate 8 wants run's unit
- drop two guards that stop nothing
- **flavors**: one unit name chora-NAME.service per plugin
- one fork check, one list of host variables
- **choractl**: drop supervise and the spawn alias

## v0.7.0-rc.2 (2026-10-25)

### Feat

- remove program start from the core
- **ipc**: fix the capabilities of a client socket (D33, D40)
- draw text nodes with cosmic-text and CHORA_FONT_DIRS
- add the text node kind with its text and font props
- **choractl**: supervise and restart a plugin without the core (T5)
- **choractl**: run starts a program with an activation token (T4)
- **ipc**: give every failure kind a stable error code
- **ipc**: negotiate the protocol version in hello (D29)

### Fix

- **tests**: gate 4 says hello as agent from the container
- **tests**: use the private mesa copy of the gates only where it exists
- **text**: draw no text instead of panicking without any font
- **ipc**: answer the protocol code for any protocol above u32
- **ipc**: keep and restore client socket capabilities on create
- **choractl**: one supervise per name, whole pid files, no orphan
- **choractl**: run and restart on a login with user systemd
- **text**: whole raster pixels, bounded caches, fallback last
- **scene**: keep font_size between 1 and 1024
- **ipc**: refuse activation tokens on client sockets, code 9
- **tests**: gate 4 follows the client socket rule of T3
- **text**: a space is laid out, not drawn, and stays quiet
- **text**: draw a glyph with a fallback font when none inks it
- **choractl**: the spawn alias warns on stderr again
- **choractl**: run never starts twice nor prints pid 0
- **query**: give rects only to what the renderer draws
- premultiply color glyphs and default text color to white
- keep text rasters per scale and cap the raster by bytes
- **choractl**: keep the MCP tools and the wait correct under load
- **choractl**: harden the MCP wait cancel and tool list retry
- **battle**: keep detached probe pids in a file across subshells
- **choractl**: end supervise without a stale pid file or orphan
- **ipc**: expire unused activation tokens without new issuance
- **run**: start programs without blocking, noise or leaked tokens
- **ipc**: answer code 6 at the camera and image cache limits
- **ipc**: answer code 6 at every limit site of the core
- **ipc**: send features in hello and test the protocol warning

### Refactor

- **choractl**: serve MCP on rmcp with parallel requests

## v0.7.0-rc.1 (2026-10-24)

### Feat

- **safety**: the owner budget decides who the watchdog may cut
- **safety**: injection mark and a fixed revoke binding for agents
- **inject**: commit texts into the focused text input
- **ipc**: described method exports and methods.list
- **tiling**: answer node requests for focus, move, close
- **scene**: requests for nodes of other owners (D31)
- **inject**: pointer_move takes a node and a point inside it
- **events**: snapshot answer of subscribe and per-connection seq (D32)
- **query**: JSON filters next to the selector string (D32)
- **core**: last words on a faulting signal
- **plugins**: come back when the core closed the connection
- **voice**: keep the agent in a conversation loop, ship the Gufo quadlets
- **xp**: export methods that do what the clicks do
- **tiling**: export window methods next to the layout methods
- **inject**: add a dry run that checks events and sends nothing
- **inject**: keep a held button down past the request
- **choractl**: wait for a selector to match or to be gone
- **ipc**: add the screen rects of each node to queries
- **ipc**: mark the focused surface and node in queries

### Fix

- **plans**: mark 0.7-then-battle.sh executable
- **plans**: reset crash checks per call and stop on empty tasks in 0.8b
- **scene**: reserve the injection mark id instead of skipping it
- **scene**: keep a client node that holds the injection mark id
- **ipc**: reject unknown keys in exports and document methods.list
- **tiling**: forget a moved window only when the core applied it
- **inject**: node point needs the node box and stays on its output
- **safety**: the disconnect pause covers only its own client
- **ipc**: let a refused client read its error line
- **battle**: feed every open finding to a fix agent
- **battle**: wait for the WM handoff after xp drags and parse the first finding
- **tiling**: mark urgent instead of focusing a hidden window
- **probe**: wake the window loop on close with --exit-on-close
- **choractl**: name the taken socket in a clients create error
- **battle**: read the compact pid of spawn and mark the friction item
- **choractl**: print the answer of spawn as one JSON line
- **ipc**: cut query rects to clips, meshes and missing assets
- **battle**: write the summary of battle.sh in English
- **battle**: mark the battle scripts with a shebang executable
- **choractl**: let clients up go on past a name in its grace time
- **protocols**: drop early surface requests beyond 64
- **headless**: cap the live virtual outputs at 16
- **transaction**: re-check the remove target at the end of a wait

## v0.6.0 (2026-10-19)

### BREAKING CHANGE

- choractl rejects view and plugin names outside
lowercase letters, digits, _, - and ., from a letter.
- hello refuses names outside lowercase letters,
digits, _, - and ., starting with a letter, at most 64 bytes.

### Feat

- let the admin socket release a camera
- name the failed command of a transaction in the error
- add choractl clients up for sockets and grants from YAML
- carry the pid and the token in surface.created
- **ipc**: let input.grab-any lift the modifier rule of grabs (D45)
- open a Wayland socket per client name (D33)
- change grants and make client sockets on an admin socket (D40)
- **xp**: ask Confirm File Replace before a drop replaces
- tell window managers when the keyboard focus moves
- **tiling**: manage only the outputs of --output
- **xp**: open desktop menus with Shift+F10 and the Menu key
- **xp**: drop desktop icons into a folder of Explorer
- **xp**: hand a window dragged out of XP's outputs to tiling
- fix the name of a client by a client socket (D33)
- **xp**: underline and take XP's access keys in context menus
- **xp**: move in Explorer's menu bar with Left and Right
- **xp**: open the system menu with Alt+Space
- **xp**: add Home and End to menus, start lists on their value
- **tiling**: claim the work areas of the outputs
- route a new toplevel to the window manager of a claim (D55)
- **xp**: open the start menu with Ctrl+Escape
- hand a surface over with remove and to, in place of give
- **xp**: work the start menu with the keyboard
- **tiling**: give the focused window away, and take given ones
- hand a surface to another client with give (D55)
- **xp**: work the context menus with the keyboard
- **tiling**: keep tags and layouts over a restart with resume
- **xp**: turn the cube with Super and a drag of the pointer
- **xp**: switch windows with XP's Alt+Tab
- **core**: give a restarted plugin its nodes back with resume
- **xp**: scroll Explorer with the mouse wheel
- **core**: end a key grab when one of its modifiers goes up
- **core**: grab pointer buttons with modifiers
- **core**: send modifiers with pointer events and scrolls
- **core**: give the keyboard focus to nodes of plugins
- **core**: run sessions headless on virtual outputs
- **xp**: let the settings dialogs wobble when they move
- **xp**: select desktop icons with XP's selection rectangle
- **xp**: open XP's menus of desktop icons and start menu entries
- **xp**: add XP's Display Properties with its five tabs
- **xp**: open XP's system menu from the title bar and task button
- **xp**: resize windows from the edges and corners of their frame
- **xp**: wobble with KWin's model on a finer grid
- **xp**: link Shared Documents in Explorer when it exists
- **xp**: underline the letters of Explorer's menu bar
- **xp**: open the places of the start menu in Explorer
- **xp**: show the Recycle Bin in Explorer and empty it
- **xp**: show My Computer with its drives in Explorer
- **xp**: list a folder in XP's Tiles view
- **xp**: draw XP's Explorer window for folders
- **xp**: add XP's Date and Time Properties
- **xp**: add XP's Taskbar and Start Menu Properties
- **xp**: open XP's context menus with the right button
- **xp**: draw XP's dialog windows with their Luna controls
- **xp**: show XP's icons on the desktop
- **xp**: play XP's sounds at logon, log off and for balloons
- **xp**: put XP's own flag on the start button when installed
- **xp**: show XP's Bliss as the wallpaper when it is installed
- **xp**: take XP's own fonts before the system fonts
- **xp**: show the icon of a balloon in the notification area
- **xp**: show notifications as XP balloon tips
- **xp**: open the Control Panel as a menu of the start menu
- **xp**: show XP's Log Off and Turn Off dialogs
- **xp**: open All Programs as cascading menus of the desktop files
- **loader**: put a nested GNOME on the void screen (wip)
- **tiling**: leave windows of given app ids to other clients
- add the loader prototype with walk and donut (wip)
- **voice**: add a prototype of the voice agent of M6 (WIP)

### Fix

- **showcase**: name the new direction of snake.py turn
- **showcase**: mark deck.py and cards.py executable
- **ipc**: show a client socket only its own nodes and windows
- **ipc**: give focus_node of a client socket only after a press
- **ipc**: keep a client socket to its own windows
- **ipc**: refuse camera commands on a client socket
- **ipc**: refuse screen:* parents on a client socket
- **showcase**: read the output of cards.py from CARDS_OUTPUT
- **showcase**: give the injected keys a name of their own
- **tests**: keep the login settings out of xp.sh
- **xp**: open no dialog for an empty trash drop
- **xp**: keep symbolic links in a copy and end the loop
- **xp**: treat only an executable desktop file as a launcher
- **xp**: do nothing for a surface of another window manager
- **tiling**: pass an ignored window on, do not reserve it
- **input**: end grabs without a modifier at their key up and revoke
- **showcase**: let the user grant, not the demos
- count the view- prefix in the view name check
- let events.subscribe know its event names
- block the emergency signal in the images worker
- keep one name rule in chora-scene, checked by choractl
- answer with the status of the core in the headless session
- require a name in hello, like every other name
- read the cap from the core and name the right socket
- wait for a freed socket place with turns, not hopes
- give shm files of the socket tests a per-call number
- find an existing socket in clients up by error data
- make the jq call in the activation step one command
- name the connection closed for an overflowed output
- drop the input injection of the deck
- change only the grants that differ in clients up
- talk to chora, not to sway, in the activation step
- guard every output change with the logical bound
- make the showcase grants and arguments work
- keep one activation token per surface, before the first map
- make clients up a declaration of state
- bound the logical size of an output on every path
- move the eaten food in the snake stress mode
- check the physical side of an output spec too
- size an image without a size by its file in a node capture
- reload the nodes of a file on a second assets.put
- **tiling**: focus the next window after the focused one exits
- use a kept activation token only within 10 s and with the focus
- let a client socket grab popups, hold pointer on focus
- **xp**: send desktop icons back after a drop on a program window
- forget the Wayland clients of a client socket that went
- grab a popup of a client socket only with its last press
- give a client socket one token per press, with focus
- drop the queued frames of a Wayland client that a revoke kills
- **tiling**: send no focus while another window manager holds it
- keep idle inhibit and idle notify for sandboxed clients
- **xp**: draw titles with one stroke when the font has no bold face
- **xp**: paint the taskbar in XP's blue for the buttons
- take activation tokens of client sockets only after input
- **kit**: show a launcher with OnlyShowIn for the current desktop
- refuse overlay layer and exclusive keyboard to sockets
- **kit**: ignore empty and relative XDG data directories
- **xp**: remove the part file of a failed replace
- **tiling**: pass on a window when tiling has no output
- disconnect the Wayland clients on revoke and reset
- **tiling**: take the focus back when the pointer returns
- keep State::new within the line limit of clippy
- hide security context and idle protocols from sockets
- **xp**: draw window titles crisp and heavy, as XP does
- **xp**: let Escape answer No to Confirm File Delete and Replace
- **xp**: ask Confirm File Replace for each clash of a drop in turn
- cap the Wayland clients of a client socket at 16
- offer no session lock or window list on a client Wayland socket
- **xp**: replace a file with one rename, not a delete and a move
- **xp**: drop nothing on a file item of an Explorer window
- **tiling**: send the focus again on a click on the focused window
- **xp**: leave the focus to tiling after a hand-over
- **xp**: ask before a drop on the Recycle Bin trashes the files
- **choractl**: resolve a symlinked socket before the admin socket
- **tiling**: give the windows away when the last output goes
- **xp**: keep the claim of chora-xp through a restart with resume
- end the focus of a node only on a click on an on-demand layer
- **xp**: let go of the focus when another window manager takes it
- **xp**: give the keyboard from the desktop to a window
- **xp**: drop desktop icons on folder items and on the Recycle Bin
- **xp**: reserve the windows of chora-xp across a Restart
- let a click on a layer surface end the focus of a node
- check for a lost last claim once per transaction
- close the gaps of client sockets from the security review
- **xp**: show a selected desktop label above the icons below it
- **xp**: keep a start menu entry lit while its context menu is open
- **xp**: go Up from My Computer and the Recycle Bin to the desktop
- **xp**: clear the leftovers of review round 16 in the menus
- **xp**: take Alt+Space only while a window of XP has the focus
- **xp**: close open menus for Ctrl+Escape and Alt+Space
- let null remove a claim
- route new toplevels only to live window managers
- end the repeat of a node key when another key goes down
- end a reservation when a keep_nodes receiver goes
- refuse a resume while the name is connected
- **xp**: keep Log Off and Turn Off out of the start menu keys
- **xp**: take Alt+Escape only while the switcher shows
- repeat a held key for a node as for a surface
- **xp**: close a context menu when the focus moves
- **xp**: remove the scene on SIGTERM and SIGINT
- **xp**: order, cancel and close XP's Alt+Tab as XP does
- **xp**: keep minimized windows and the taskbar order over a restart
- **xp**: keep the cube still on a Super click without a drag
- let Shift not end a held key grab
- **xp**: start fresh after a crash, rebuild more simply
- **xp**: draw the wobble from whole steps moved on to the present
- **xp**: keep the windows over a restart with resume
- **xp**: keep Recycle Bin Tasks in an empty Recycle Bin
- **xp**: handle the queued moves of the pointer as the last one
- **xp**: wobble in steps of any length with a drag of their length
- **xp**: show the icons of file types on the desktop
- **xp**: give programs without an icon XP's icon of a program
- **plugins**: set the new resume field of hello
- **xp**: keep nine-slice images at scale 1 after a scale change
- **xp**: draw tiles at their size on a scaled output
- **xp**: wobble in whole steps of 10 ms at any rate of the pointer
- **xp**: give the focus to the top window when a dialog closes
- **xp**: keep the selection on a right click on one of its icons
- **xp**: keep the cells of desktop icons when another one goes
- **xp**: keep the windows and Explorer windows over a restart
- **xp**: draw a control again only when its look changed
- **xp**: advance the wobble when the pointer moves the window
- **xp**: move windows back into the work area after a scale change
- **xp**: render pictures at the scale of the outputs
- **core**: map an unmapped toplevel again without its old configure
- **core**: keep a clicked layer surface focused when a toplevel goes
- **xp**: keep the grabbed point of a wobbly window under the pointer
- **xp**: scroll Explorer without new nodes for the whole page
- **core**: resize lock surfaces on each output change
- **core**: treat an unmapped toplevel as a destroyed one
- **core**: take the keyboard from a stuck layer surface
- **ipc**: shut a closed connection down at once
- **core**: end focus returns when a transaction moves the focus
- **ipc**: keep a connection until its process answers are out
- **api**: tell the version of the core in the answer to hello
- **ipc**: limit the connections of one process, say why
- **api**: accept cameras only for outputs that the core knows
- **api**: limit the meta data of a node and keep too large answers
- **api**: name the wrong key of a transform in its error
- **ipc**: answer a request line that is too long before the close
- **xp**: drop the effects only when the watchdog names chora-xp
- **core**: copy decoded pixels into their buffer on the worker
- **core**: let the watchdog blame the client that slows the frames
- **core**: decode PNG files on a worker thread
- **core**: give the pointer in surface units on a scaled node
- **core**: let a popup grab with the serial of a key release
- **render**: draw the color of a rect with straight alpha
- **xp**: find the XP icon themes by the start of their name
- **xp**: keep a maximize that comes before the first commit
- **xp**: move the windows of a removed output to another one
- **tiling**: say where the old command line options went
- **tiling**: reject a key bound twice and a step that is no number
- **plugins**: keep the rest of a transaction when a window closed
- **core**: answer a gone surface with its own code at the check too
- **core**: let the focus move while an input method has it
- **core**: open only regular files that a client names
- **tiling**: keep one tag shown when its view is toggled off
- **loader**: show the donut whenever a part of it is in view (wip)
- **loader**: keep an opaque backdrop under the walk (wip)
- **release**: main check without SIGPIPE, and RPM names without tilde
- **core**: the emergency reset ends transactions without effect
- **ipc**: start the next transaction when a connection closes

### Refactor

- join RpcError, group kit imports, name the reload ticket
- move the plugin connection skeleton into chora-kit
- **xp**: drop the unused panel sheet of Exposé
- name the socket path in the warning, one lookup
- **kit**: keep genie and parse private, name launcher fields
- **kit**: let Icons::new take any list of theme names
- **tiling**: forget a window that leaves tiling in one place
- **kit**: move the desktop entry parser into chora-kit
- give each connection one socket kind, one layer filter
- **kit**: move the icon theme lookup and XDG dirs to chora-kit
- **kit**: move the magic lamp into chora-kit
- **kit**: move the wobbly model into a new crate chora-kit
- **xp**: use one key type and one wrap for both kinds of menus
- **xp**: give each menu one helper and one branch at the end
- **xp**: reopen a kept Explorer window without clones
- **tiling**: send the whole meta of a node when it changes
- **xp**: clear the small leftovers of review round 10
- **xp**: find the icons of desktop files once in the scan
- **xp**: clear the small leftovers of review round 8
- **core**: drop the resumed answer of hello
- **xp**: tidy the comments and tests of the review
- **xp**: scroll Explorer by wheel and thumb as by clicks
- **xp**: find the top window for a closed dialog as elsewhere
- **xp**: drop the check of insets in the rescale
- **xp**: keep flat files apart from sharp ones in the theme
- **xp**: simplify the changes of the outputs and the scale
- **xp**: mark the restart file as a stopgap
- **xp**: say when the desktop icons keep their cells
- **core**: borrow the focused surface instead of copying it
- **xp**: tidy the pages after the scroll of Explorer
- **xp**: let the empty view take clicks without a white picture
- **xp**: scroll the view of a page in one group
- **core**: keep the layer focus unless a surface gets the focus
- **scene**: end focus returns of a focus command in set_focused
- **scene**: add a surface node without a check for an old one
- **core**: resize no lock surfaces when an output goes
- **core**: check a clicked layer surface in one line
- **xp**: turn the cube with the easing of chora-scene
- **core**: end all deadlines in one place on a time of the state
- **core**: split the core into a library and a binary

## v0.5.4 (2026-10-14)

### Feat

- **api**: tell in pointer.entered whether the pointer moved
- **tiling**: follow the user experience of awesome

### Fix

- **process**: close the waiting connections of an exited plugin
- **ipc**: free grabs and methods at the end of a stream
- **tiling**: a fullscreen window follows the raise, not the pointer
- **tiling**: a click raises, and fullscreen follows the focus
- **tiling**: raise on a click or a binding, not on pointer focus
- **tiling**: the findings of the second review round
- **tiling**: the findings of the review
- **process**: restart a plugin after its connections closed

## v0.5.3 (2026-10-13)

Milestone M4.2: the API covers what the first test round on real hardware
missed. `input.ungrab` ends a grab, and `input.key` tells the release of a
grabbed key. Each output has an overlay root `screen:NAME` for a HUD that
stays in place when the camera moves. `process.spawn` with `capture` keeps
the output of a program for `process.wait`, and `choractl spawn --capture`
prints it. `output.configure` sets the scale and the transform of an
output. A mesh can bend into 3D with a depth per point, and a point behind
the eye hides only its triangles. Nested, chora takes the keymap of the
host session. Three review rounds went over these changes before the
release.

### Known issues

- Nobody has run chora on real hardware for long yet: the 8-hour session
  and the 60 Hz goal on Intel graphics are untested.
- foot can die with `wl_shm_pool: offset is too large` at output scale 3
  (#14); a reproduction is open.
- The keymap of the host is untested in CI, whose host has no keyboard.
- A grant protects against mistakes of agents, not against hostile code of
  the same user (D40). The grant `process.spawn` has no allowlist yet
  (D42).
- grim and other `wlr-screencopy` clients turn the picture of a rotated
  output once more. Screencopy offers shm buffers only, and pictures have
  no cursor.
- Layers render again in every frame, they have no depth test and no
  mipmaps.

### Feat

- M4.2 test round: grab releases, overlays, process output, mesh depth, host keymap

### Fix

- second review round on M4.2
- review findings on M4.2

## v0.5.2 (2026-10-13)

Fixes from the first intensive test round on real hardware. Nested, chora
releases the keys that are down when its window loses the focus, and no
longer floods the log with `BAD_SURFACE` in every frame of an animation.
chora-tiling and chora-xp keep their windows in place when a camera moves
or zooms, with the new `default_work_area` of an output. A transaction on
a window that closed in the meantime fails with the new error code 2, which
plugins ignore. The new event `layers.dropped` and the field `left_out` of a
capture name the layers that the pixel budget of a frame leaves out of the
screen. A client that cleared the focus may give it back. The core enables
`GL_OES_standard_derivatives` for shaders that need it. Releases now start
by themselves when a new version reaches main. Two review rounds went over
these changes before the release.

### Known issues

- Nobody has run chora on real hardware for long yet: the 8-hour session
  and the 60 Hz goal on Intel graphics are untested.
- foot can die with `wl_shm_pool: offset is too large` at output scale 3
  (#14); a reproduction is open.
- Nested, chora uses its own keymap, the US layout by default; set
  `XKB_DEFAULT_LAYOUT` (#12).
- A grant protects against mistakes of agents, not against hostile code of
  the same user (D40). The grant `process.spawn` has no allowlist yet
  (D42).
- grim and other `wlr-screencopy` clients turn the picture of a rotated
  output once more. Screencopy offers shm buffers only, and pictures have
  no cursor.
- Layers render again in every frame, and they have no depth test.

### Fix

- findings of the second review round of the test round
- findings of the review of the test round
- **xp**: keep the taskbar, the wallpaper and maximized windows in place under a camera
- findings of the test round after v0.5.1

## v0.5.1 (2026-10-13)

Each release now has a package for Arch Linux next to the RPM. The fixes
come from a review of M4.1: `plugins.restart` follows the old process
through a pidfd, refuses a second restart of the same name and keeps empty
arguments. `choractl mcp` sends a call with an effect only once after a
lost connection, and reserves its space again on the new one. The startup
command keeps the stdin of chora again, as in v0.4.0.

### Feat

- build a package for Arch Linux with each release

### Fix

- findings of the second review round
- findings of the review of M4.1

## v0.5.0 (2026-10-13)

Agent feedback (milestone M4.1), from the issues of the first MCP
sessions. The tools of `choractl mcp` carry the schemas of the commands and
the input events, and an agent subscribes to events and waits for them. It
hands its own shaders and images to the core with `assets.put`, reserves
space at the edges of an output with `output.reserve`, and moves cameras
with an animation. With the new grant `process.spawn`, it starts programs
and restarts plugins. Rects have rounded corners and borders, every node
clips, and shader nodes draw a shader that can read other nodes. The last
step of `tests/agent.sh` is the exit test: an MCP client draws a panel with
two buttons, and a click on a button moves the camera.

### Breaking changes

- `tiling.set_layout` takes no `reserve` any more; use `output.reserve`.
- `animation.done` has an optional `node` and an optional `output`.

### Known issues

- Nobody has run chora on real hardware yet: the tty backend, the effects
  on a GPU and the 60 Hz goal on Intel graphics are untested.
- A grant protects against mistakes of agents, not against hostile code of
  the same user, which can call `grants.set` itself (D40). The grant
  `process.spawn` has no allowlist yet (D42).
- `plugins.restart` needs the process of the plugin in `/proc` of the core,
  and starts its command in the namespace of the core.
- grim and other `wlr-screencopy` clients turn the picture of a rotated
  output once more: the core sends it as it shows.
- Screen sharing through `xdg-desktop-portal-wlr` is untested. Screencopy
  offers shm buffers only, no dmabuf, and pictures have no cursor.
- `input.inject` types through the first layout of the keymap, with Shift
  or AltGr at most.
- Layers render again in every frame, and they have no depth test.
- The XP flavor of v0.3.0 is unchanged and keeps its limits.

### Feat

- start programs and restart plugins behind the grant process.spawn
- **scene**: move cameras with a duration and an easing
- let IPC clients reserve space at the edges of an output
- let clients hand image and shader files to the core with assets.put
- **choractl**: give apply_commands and inject_input the schemas of their types
- **choractl**: subscribe to and wait for events over MCP
- rounded rects with borders, clips and shader nodes

### Fix

- **scene**: let a camera ignore the z axis
- **choractl**: answer invalid tool arguments with isError

## v0.4.0 (2026-10-12)

The agent interface (milestone M4): an agent sees and changes the screen.
`choractl mcp` serves the API as MCP tools, with `describe_screen` for a
screenshot and the scene in words. Plugins export methods, such as
`tiling.set_layout`. The user grants screenshots and input with
`choractl grant`. Views load plugins and nodes from strict YAML. grim and
screen sharing capture through `wlr-screencopy` and
`ext-image-copy-capture`, and sandboxed apps do not see the privileged
protocols. Nested, `tests/agent.sh` checks it, and its last step is the
exit test: an MCP client describes the screen, calls `tiling.set_layout`,
and sees the new layout in the next screenshot.

### Known issues

- Nobody has run chora on real hardware yet: the tty backend, the effects
  on a GPU and the 60 Hz goal on Intel graphics are untested.
- A grant protects against mistakes of agents, not against hostile code of
  the same user, which can call `grants.set` itself (D40).
- grim and other `wlr-screencopy` clients turn the picture of a rotated
  output once more: the core sends it as it shows.
- Screen sharing through `xdg-desktop-portal-wlr` is untested. Screencopy
  offers shm buffers only, no dmabuf, and pictures have no cursor.
- `input.inject` types through the first layout of the keymap, with Shift
  or AltGr at most.
- Layers render again in every frame, and they have no depth test.
- The XP flavor of v0.3.0 is unchanged and keeps its limits.

### Feat

- **choractl**: serve the API as MCP tools over stdio
- **choractl**: load, save and unload views from strict YAML
- hide privileged protocols from sandboxed clients
- capture outputs and toplevels through ext-image-copy-capture
- let grim and screen sharing capture outputs through wlr-screencopy
- inject input after a grant of the user
- capture outputs and nodes after a grant of the user
- **tiling**: switch between master and stack, columns and rows
- let plugins export methods to other clients

### Fix

- **choractl**: keep views in their directory and MCP to the spec
- **tiling**: keep running when the call of an answer ended
- pace screencopy frames and bound captures and injections

## v0.3.0 (2026-10-11)

The showroom (milestone M3): layers with meshes, 4x4 transforms and GLSL
effects, animations in the core, image nodes and pointer events over
nodes. The new XP flavor shows them off: a desktop in the style of
Windows XP with wobbly windows, a magic lamp minimize and a desktop cube.
Log in with the session "chora (Windows XP)". No file of Microsoft ships
with it.
Nested, `tests/nested.sh` and `tests/xp.sh` check the features.

### Known issues

- Nobody has run this release on real hardware yet: the tty backend,
  effects on a GPU and the 60 Hz goal on Intel graphics are untested.
- Layers render again in every frame, even when nothing in them changed.
- Layers have no depth test: they draw in the order of their siblings.
- A rotated or projected layer takes the input over its bounding box.
- Every client of the user can take the keyboard as an input method and
  configure the outputs, as in sway; sandboxed clients lose this with
  `wp-security-context-v1` in M4.
- Taskbars of other programs never see a toplevel as minimized.
- The XP flavor is a demo, not a daily driver:
  - windows have no resize borders, and there is no Alt+Tab;
  - fullscreen requests are ignored;
  - the cube shows only the first output;
  - windows that draw their own frame cannot be moved;
  - All Programs opens the launcher;
  - while the cube turns, lifted windows of the far side can hide behind
    the near side.

### Feat

- **xp**: light up buttons and menu entries under the pointer
- tell plugins when the pointer enters and leaves a node
- **xp**: leave the frame to windows that draw their own
- **xp**: wobble windows, minimize through a lamp, turn a desktop cube
- tell plugins who draws the frame of a window
- shade layers with GLSL effects that reload on change
- report size changes, and read events with a timeout
- **xp**: add the XP flavor, its session and its RPM files
- **xp**: add chora-xp, a window manager in the style of Windows XP
- **xp**: add the theme of the XP flavor
- **safety**: remove effects and reset when frames stay slow
- render layers through meshes with full 4x4 transforms
- tell plugins about presses and drags over nodes
- animate node properties in the core
- add image nodes with nine-slice scaling

### Fix

- harden the watchdog, layers, images and effects
- **winit**: pace frames without damage instead of rendering at once again
- **input**: send pointer.moved only for a new position
- **ipc**: let plugins grab Ctrl+Alt keys outside the safety layer
- **scene**: read nodes without effects back from their JSON
- **xp**: read queries without effects, events through the client

## v0.2.0 (2026-10-11)

The session on real hardware (milestone M2): log in to chora with the
tiling flavor, with a panel, a launcher, a lock screen and X11 programs.
Nested, `tests/nested.sh` checks waybar, fuzzel, swaylock, wlrctl and
wlr-randr.

### Known issues

- Nobody has run this release on real hardware yet: the tty backend, the
  login session and the tiling flavor are untested there.
- On the tty backend, a mode change through wlr-output-management that
  fails at the next frame is not reported to the client.
- Every client of the user can take the keyboard as an input method and
  configure the outputs, as in sway; sandboxed clients lose this with
  `wp-security-context-v1` in M4.
- Taskbars never see a toplevel as minimized.

### Feat

- change modes and turn outputs off on the tty backend
- implement wlr-output-management in the core
- **tiling**: start a launcher with MOD+d
- show the new buffers of a transaction with it
- **tiling**: close a window when a taskbar asks
- add wlr-foreign-toplevel-management for taskbars
- report presentation times
- add a login session with the tiling flavor
- add input, idle, scale and toplevel protocols
- **scene**: add the surface.requested event
- run on real hardware, with panels and a lock screen

### Fix

- **session**: give X11 programs DISPLAY and drop a stale environment
- **tty**: keep removed output globals, unique names, dark off outputs
- **lock**: confirm a lock only with frames rendered under it
- keep the GL flip of the winit window out of the output
- **layers**: announce the work area when a panel goes away
- **input**: switch the VT by the modified keysyms
- harden the lock screen, popup grabs and the tty backend
- refresh the pointer after transactions and keep re-map sizes
- **scene**: validate cameras strictly and limit them to 64
- **render**: scale windows by the scale of their node and camera

## v0.1.0 (2026-10-11)

The nested proof of concept (milestone M1): run `chora -- chora-tiling`
inside another Wayland session.

### Known issues

- A camera or a node with a scale other than 1 moves windows, but does not
  resize them.
- The number of cameras has no limit, and a camera change does not update
  the surface under the pointer until the pointer moves.
- A window that unmaps and maps again loses the size that its plugin set.

### Feat

- show the scene on each output through a camera
- grab input for popups, so that menus close on a click outside
- start the emergency reset on SIGUSR1
- add the chora-tiling reference plugin
- add choractl with query, apply, events and outputs
- allow quoted selector values with spaces
- add the nested compositor core with ipc and transactions
- add the scene model with ownership rules

### Fix

- **transaction**: let late surfaces catch up with their newest configure
- **ipc**: send the events of a close in the same loop iteration
- **scene**: leave surfaces out of the node limit
- keep the surface node when a client maps it again
- **safety**: start the startup command before SIGUSR1 is blocked
- **ipc**: close connections that send no hello within 5 s
- **scene**: limit the scene to 16,384 nodes
- **transaction**: stop waiting for surfaces that missed a deadline
- **ipc**: answer waiting transactions after the end of the stream
- **safety**: block SIGUSR1 before any thread starts
- send surface.destroyed only for surfaces with a node
- **scene**: reserve ids of the form s1 for surfaces
- give choractl clear errors and a clean end of events
- keep the tiling plugin consistent with the scene
- let the user type after an emergency reset
- keep host session variables from the startup command
- harden the ipc socket and the surface lifecycle
- limit waiting transactions and copy the scene once per check
- block pointer input only where the blocking node is
- skip nodes with a scale that is not positive
- reject negative sizes and numbers that are not finite
- let the ipc client skip unknown events and null-id errors
- reject size and color on nodes that are not rects
- attach only surfaces without an owner
- clear the properties of detached surfaces
- report every detached surface exactly once
- walk the scene without recursion
- install the license file also without docs
- create the rpm output directory before the build
- catch the deprecated GPL-3.0 identifier in the hook

### Refactor

- **render**: chain the hit test of surface_under
- simplify event filters, IPC reads and hit tests
- drop dead state fields and a hand-made rounding helper
- drop duplicated serde defaults and event names

### Perf

- **scene**: keep the nodes in a persistent map
- draw only damage and throttle frames of hidden windows
