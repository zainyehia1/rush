# rush | Custom UNIX Shell in Rust

A lightweight, POSIX-inspired UNIX shell built from scratch in Rust. Features include command-line parsing with quote and escape handling, stdout/stderr redirection and appending, shell and environment variable expansion, background job processing, persistent history, and tab completion with custom completion script support.

The `rustyline` crate was used to manage tab completions, history navigation, and to support the interactive REPL.

---

## Features
* **Interactive REPL:** Continuously reads and executes commands until `exit` is called.
* **Built-in Shell Commands:** 
    - `echo`: Prints arguments with support for redirections.
    - `pwd`: Prints the current working directory path.
    - `cd`: Change directory (supports relative paths, absolute paths, and `~` expansion).
    - `type`: Identify whether a given command is a shell builtin or an external executable.
    - `history`: View command history, with `-r`, `-w`, and `-a` flags for reading/writing/appending history files.
    - `declare`: Handles local shell variable assignments and lookups via `-p`.
    - `jobs`: List background tasks and their status.
    - `complete`: Registers external custom programmable tab-completion hooks.
    - `exit`: Exits the REPL (saves history to `$HISTFILE` if set).
* **Stream Redirection:** 
    - `>` / `1>` — overwrite stdout to file
    - `2>` — overwrite stderr to file
    - `>>` / `1>>` — append stdout to file
    - `2>>` — append stderr to file
* **Variable Expansion:**
    - Support for `$VAR` and braced `${VAR}` syntax.
    - Expands both environment variables and shell-local variables set with the `declare` builtin.
* **Background Jobs:**
    - Append `&` to any external command to run it in the background.
    - Job IDs and PIDs are printed on spawn.
    - Finished jobs are reaped and reported at next prompt.
* **Tab Completion:**
    - Commands complete from builtins and everything on your `$PATH` environment variable.
    - Arguments fall back to filename/path completion.
    - Custom completion scripts can be registered per-command with `complete -C`.
* **History:**
    - History is loaded from `$HISTFILE` (if set) on start and saved on exit.
    - `history -r <file>` reads history from a file.
    - `history -w <file>` writes history to a file. (Overwrites)
    - `history -a <file>` appends history to a file.
    - Up/down arrow history navigation supported (via `rustyline`).
---
