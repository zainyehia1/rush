# rush | Custom UNIX Shell in Rust

A lightweight, POSIX-inspired UNIX shell built from scratch in Rust. Features include command-line parsing with quote and escape handling, stdout/stderr redirection and appending, shell and environment variable expansion, background job processing, persistent history, and tab completion with custom completion script support.

The `rustyline` crate was used to manage tab completions, history navigation, and to support the interactive REPL.

---

## Features
* **Interactive REPL:** Continuously reads and executes commands until `exit` is called, with a prompt showing the current directory.
* **Built-in Shell Commands:** 
    - `echo`: Prints arguments with support for redirections.
    - `pwd`: Prints the current working directory path.
    - `cd`: Change directory (supports relative paths, absolute paths, and `~` expansion).
    - `type`: Identify whether a given command is a shell builtin or an external executable.
    - `history`: View command history, with `-r`, `-w`, and `-a` flags for reading/writing/appending history files.
    - `declare`: Set local shell variables via `=` and inspect them via `-p`.
    - `jobs`: List background tasks and their status.
    - `complete`: Register custom tab-completion scripts per command (`complete -C <script> <command>`)
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
    - Up/down arrow history navigation (via `rustyline`).
---

## Installation
`rush` requires version 1.85.0 of `Rust` and a Unix-like system (Linux, macOS, WSL).
```bash
git clone https://github.com/zainyehia1/rush
cd rush
cargo install --path . 
```
---

## Usage
It works like a standard shell. Run commands normally.
```sh
~/workspace/rush$ echo hello world
hello world
~/workspace/rush$ type echo
echo is a shell builtin
~/workspace/rush$ type cat
cat is /usr/bin/cat
~/workspace/rush$ pwd
/home/user/workspace/rush
~/workspace/rush$ declare greeting=hello
~/workspace/rush$ declare -p greeting
declare -- greeting="hello"
~/workspace/rush$ declare species=human
~/workspace/rush$ echo $greeting $species
hello human
~/workspace/rush$ echo $HOME
/home/user
~/workspace/rush$ hi
hi: command not found
~/workspace/rush$ history
1 echo hello world
2 type echo
3 type cat
4 pwd
5 declare greeting=hello
6 declare -p greeting
7 declare species=human
8 echo $greeting $species
9 echo $HOME
10 hi
11 history
~/workspace/rush$ history 5
8 echo $greeting $species
9 echo $HOME
10 hi
11 history
12 history 5
~/workspace/rush$ sleep 25 &
[1] 768114
~/workspace/rush$ sleep 20 &
[2] 768115
~/workspace/rush$ sleep 15 &
[3] 768116
~/workspace/rush$ jobs
[1]   Running                 sleep 25 &
[2]-  Running                 sleep 20 &
[3]+  Running                 sleep 15 &
# after the jobs finish:
~/workspace/rush$ jobs
[1]   Done                    sleep 25
[2]-  Done                    sleep 20
[3]+  Done                    sleep 15
~/workspace/rush$ cd ..
~/workspace$ cd rush/
~/workspace/rush$ echo hello world > output.txt
~/workspace/rush$ cat output.txt
hello world
~/workspace/rush$ echo bye >> output.txt
~/workspace/rush$ cat output.txt
hello world
bye
~/workspace/rush$ 
```
---

## Limitations
* **No pipelines**
* **No aliases**
* **No `export`**
* **No exit codes**
* **No script execution**
* **No command substitution**
* **No arithmetic expansion**
* **No foreground jobs (fg)**
* **No brace expansion**
* **No profiles**
* **No stopped jobs**
* **No input redirection**
* **Not fully POSIX-compliant**
---

## Disclaimer
This project was based on the CodeCrafters "Build your own Shell" challenge, with some additions beyond the core stages. 