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
the log of the new run. The folder has mode 0700 and the files 0600.

The plugins (`chora-tiling`, `chora-xp`, `chora-mc`, `chora-construct`,
`chora-notify`, `chora-assistant`) write to stderr only. In a session,
systemd runs each program as a unit and stderr goes to the journal:

```sh
journalctl --user -b -u 'chora*'
journalctl --user -b -u chora-voice -f
```

## How to read a line

```
2026-11-15T10:00:02.123Z  INFO chora::ipc: refused conn=7 client="agent" code=4
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
log: a dependency can write the payload of a request at debug.

A refused IPC request logs its client and its error code (see
[API](api.md)). The codes 2 (gone surface) and 5 (unknown node) are
races of every client and log at debug.

## What a log never holds

No level logs the text that the person types or says, a reply of the
voice agent, a chat message, a coding task or the arguments of a tool
call: such lines log the length only (`turn=2 chars=31`). No level logs
a token, a key or a password. The values of settings keys such as
`llm_headers` or `*_api_key_file` show as `(hidden)`.

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
   as `token`, `secret`, `password`, `api_key` or `authorization`, and
   each string that looks like a token, with `(hidden)`. It cannot
   know every private word: remove what you do not want to share.
4. Open an issue on [GitHub](https://github.com/MArpogaus/chora/issues)
   with the notes of step 1 and attach the bundle.

`chora-report` uploads nothing. `chora-report -o FILE` writes the
bundle to FILE.
