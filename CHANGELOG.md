## v0.8.0-rc.2 (2026-10-30)

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
