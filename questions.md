# Questions for the user

Each: number, date, background, default taken.

1. 2026-11-09. D75 (f5f346f) keeps "a depth buffer in the core" rejected
   and narrows the rejected row instead of marking it superseded, as the
   charter said. Reason: sorting and splitting planes handles translucent
   pane edges, which a depth buffer does not, and Smithay's framebuffers
   have no depth attachment; dense 3D geometry belongs in clients.
   Default taken: keep it rejected. Answer "allow a depth buffer" to
   reopen it.
2. 2026-11-09. 83f9b6c marks the chora-mc methods `agent.set` and
   `notify.push` privileged in the core. Voice already asked before them;
   new is that other clients (for example an agent on a client socket)
   now need the grant `methods.privileged` to call them. Default taken:
   keep it. Answer "unprivileged" to revert.

3. ANSWERED 2026-11-09 (accepted). Repository split (`design/split.md`): four repositories
   after 1.0 in the organization `chora-desktop`: `chora` (core, API
   crates, choractl, chora-kit, chora-tiling as the reference plugin),
   `chora-plugins` (xp, mc, construct), `chora-voice`, `chora-website`.
   This amends D36 (one repository per plugin and a packaging
   repository). Default: prepare for this split; nothing is split.
   Answer "keep D36" to prepare the old plan instead.
4. 2026-11-09. Publish `chora-scene` and `chora-kit` on crates.io at
   1.0 (both names are free; `chora` is taken). Default: yes; the
   fallback is a git dependency on a tag.
5. 2026-11-09. Only you can create the empty GitHub organization
   `chora-desktop` now, so that the name stays free. Default: not done.

6. ANSWERED 2026-11-09: make it configurable; the user can raise grants
   temporarily or permanently. Since security fix (a) (the voice agent's MCP connection
   is confined with the full client socket rules, 0a3d34e and eded56f),
   the voice agent can no longer close, configure or attach other
   clients' windows directly, nor move a camera; it must ask the window
   manager (`node.requested`, D31, D58, window manager methods). Views
   with images or shaders no longer load through `load_view`. Default
   taken: keep the confinement (it is what the go for (a) asked); make
   the window manager path the documented one, fix
   `services/voice/README.md`, and check that each window manager offers
   the methods voice needs (close, move, resize). Answer "loosen" to let
   the confined agent act on foreign windows again with a grant.

## Q7 (2026-11-09): HUD and ACP chat sprint

Numbers match `design/hud-sprint.md` section 8. Each has one default
that is taken unless you answer; answer with the number and the word,
for example "7.3 hand over". The text of each default is in the design
file. Defaults on wording, key line and badge are in
`design/ux-hud-sprint.md` section 8.

7.1 Decided by you (11-09): the chat lives inside each look, drawn by
    the host and styled by the HUD folder, with one layer-surface
    field. It is not a terminal window. No answer needed.
7.2 Host: grow `chora-assistant` into the HUD host, or a new block
    `chora-hud`? Default: grow `chora-assistant`. Answer word: `new`.
7.3 Do mc and construct hand their voice HUD to the host? Default:
    no, `voice_hud: own` this sprint; the PM looks again after the user
    tests. Answer word: `hand over`.
7.4 May a typed "yes" in the chat answer a `read`, `search` or `think`
    question of an agent? Default: no, the chat answers nothing; only a
    key or a button does. Answer word: `allow`.
7.5 Does an agent with an open chat speak its results? Default: no,
    only its questions. Answer word: `speak`.
7.6 Type to the voice LLM itself in the chat, not only to ACP agents?
    Default: not this sprint; the picker lists helpers only. Answer
    word: `include`.
7.7 May a HUD folder call non-privileged plugin methods, such as
    `tiling.get_tags`? Default: no, events only; a plugin emits what a
    HUD needs. Answer word: `methods`.
7.8 Does a dropped HUD folder activate itself? Default: no, one line
    `hud: NAME` in `chora-assistant.yaml`. Answer word: `auto`.
