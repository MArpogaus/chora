# Logs and problem reports

## Where the logs are

The core `chora` and the voice agent `chora-voice` write their log to
stderr and to a log file:

| Program | File |
|---------|------|
| `chora` | `$XDG_STATE_HOME/chora/logs/chora.log` |
| `chora-voice` | `$XDG_STATE_HOME/chora/logs/voice.log` |

Without `$XDG_STATE_HOME` the folder is `~/.local/state/chora/logs`. A
file grows to 4 MiB. Then it moves to `chora.log.1`, the older files
move one number up, and three old files stay. Each start moves the file
of the last run to `chora.log.1`, so the log of a crash stays next to
the log of the new run.

Only one process writes a log file: the one that holds the lock
`chora.lock` (or `voice.lock`) in the same folder. A second `chora`,
for example a nested or a test session, logs to stderr only and leaves
the file of the first alone. `chora-voice` writes `voice.log` only when
it runs as the agent; `chora-voice --version`, `--print-config` and its
subcommands (`ask`, `tts`, ...) log to stderr only and move no file.
The folder has mode 0700 and the files 0600; a log file or a log
folder that is a symlink is not used: the program logs one warning
`no log file` and goes on with stderr only, and the symlink and its
target stay as they are.

The plugins (`chora-tiling`, `chora-xp`, `chora-mc`, `chora-construct`,
`chora-notify`, `chora-assistant`) write to stderr only. In a session,
systemd runs each program as a unit and stderr goes to the journal:

```sh
journalctl --user -b -u 'chora*'
journalctl --user -b -u chora-voice -f
```

## How to read a line

```
2026-11-15T10:00:02.123456Z  INFO chora::ipc: refused conn=7 client="agent" code=4
```

The parts are the time in UTC, the level, the target and the message
with its fields. The target of the core is the area of the program, its
module path, for example `chora::ipc` or `chora::backend::tty`. The
voice agent logs with the target `chora_voice::log`; the first word of
its message names the stage, for example `turn=2`, `llm`, `tool` or
`confirm`.

| Level | Meaning |
|-------|---------|
| `ERROR` | A part of the program stops: a thread, a panic, a failed start. |
| `WARN` | Something failed and chora goes on without it: an output, a frame, a client. |
| `INFO` | A change of state: start, end, lock, a grant, a refused request, a turn of the voice agent. |
| `DEBUG`, `TRACE` | Details for a developer. |

The default level is info. `RUST_LOG` changes it, for example
`RUST_LOG=chora::ipc=debug,info`. Below info only the targets of chora
log: a dependency can write the payload of a request at debug. The MCP
library `rmcp` of the voice agent logs warnings and errors only, at any
`RUST_LOG`: its info lines carry the payload of a notification.

A refused IPC request logs its client and its error code (see
[API](api.md)). The codes 2 (gone surface) and 5 (unknown node) are
races of every client and log at debug.

## What a log holds

No level logs the text that the person types or says, a reply of the
voice agent, a chat message, a coding task or the arguments of a tool
call: such lines log the length only (`turn=2 chars=31`). The values of
settings keys such as `llm_headers` or `*_api_key_file` show as
`(hidden)`.

A log can hold:

- at info, the command line of a program that the voice agent starts:
  `agent NAME: COMMAND` for a sub-agent and `mcp: restart with COMMAND`
  for an MCP server. A secret in such a command line lands in
  `voice.log`; pass secrets in a file or in the environment;
- in an error, the URL of a request that failed, with its query. A key
  in a URL (`?key=...`) lands in `voice.log`; send keys in
  `llm_headers`.

`CHORA_VOICE_LLM_DUMP=FILE` (see [voice](voice.md)) writes every LLM
request and reply in full, with what the person said, to FILE. It is
for debugging only; FILE is not a log file and `chora-report` does not
take it.

## How to report a problem

1. Note what you did, what you expected and what happened instead.
2. Run `chora-report`. It writes `chora-report-DATE.tar.gz` in the
   current folder with:
    - `versions.txt`: the versions of `chora`, `choractl` and
      `chora-voice`, the kernel, the system and the session;
    - `logs/`: the log files of `$XDG_STATE_HOME/chora/logs`;
    - `journal.txt`: the journal of the chora units of this boot;
    - `config/`: the settings files of `$XDG_CONFIG_HOME/chora`.
3. Read the bundle before you send it: `tar -xzf chora-report-*.tar.gz`.
   `chora-report` replaces the home path with `~`, the user name with
   `USER`, the host name with `HOST`, and the value of each key such
   as `token`, `secret`, `password`, `api_key`, `Api-Key` or
   `authorization`, with the indented lines below it, and each string
   that looks like a token, with `(hidden)`. Settings files that are
   symlinks go in as the files they point to, and only you can read the
   bundle (mode 0600). It cannot know every private word: remove what
   you do not want to share.
4. Open an issue on [GitHub](https://github.com/MArpogaus/chora/issues)
   with the notes of step 1 and attach the bundle.

`chora-report` uploads nothing. `chora-report -o FILE` writes the
bundle to FILE.
