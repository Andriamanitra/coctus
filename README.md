# CoCtus

Command line tool that allows you to play [Clash of Code](https://www.codingame.com/multiplayer/clashofcode) style problems locally on your computer.


## Usage

These commands should be enough to get you started:

```
# Download a couple of clashes by id (last part of the URL on the contribution page)
coctus fetch 52980368bdbd05abdd789a04173b57b0fdea 682102420fbce0fce95e0ee56095ea2b9924

# View information such as the id of the current problem and the directory where clashes are being stored
coctus status

# Move on to next (random) problem
coctus next

# Show the current problem
coctus show

# Select a specific problem
coctus next 52980368bdbd05abdd789a04173b57b0fdea

# Show a specific problem
coctus show 52980368bdbd05abdd789a04173b57b0fdea

# Run Python script "sol.py" against the tests of the current problem
coctus run --command "python3 sol.py"

# Compile a C program "sol.c" and run it against the tests of the current problem
coctus run --build-command "gcc -o sol sol.c" --command "./sol"
```

Use `coctus help` and `coctus <SUBCOMMAND> --help` to show all available options.

### Auto-refreshing using `entr` or `nodemon`

We recommend using a program such as [entr](https://github.com/eradman/entr) or [nodemon](https://www.npmjs.com/package/nodemon) to refresh the statement when the current clash changes and/or automatically run tests when a file is saved.
Combining `coctus` with one of these tools allows you to never need to leave your text editor while clashing.

#### Example 1: Automatically show the new problem when it changes:

This example uses `entr` (or `nodemon`) to detect changes to the file that keeps track of the current clash and run the command `coctus show`.
The `-c` flag to `entr` clears the terminal before running the command.
Press Ctrl-c to stop.

```
# Option 1: using entr (Linux only)
ls ~/.local/share/coctus/current | entr -c coctus show

# Option 2: using nodemon (PowerShell on Windows only)
nodemon --watch "$env:APPDATA\CoCtus\coctus\data\current" --exec coctus show
```

#### Example 2: Automatically run code when a file is saved

This example uses `entr` (or `nodemon`) to watch over .py files in the current directory and run `python3 sol.py` when any of them are saved to disk.
The `--auto-advance` flag automatically does the equivalent of `coctus next` when you pass all the test cases.
Press Ctrl-c to stop.

```
# Option 1: using entr (Linux only)
ls *.py | entr coctus run --auto-advance --command "python3 sol.py"

# Option 2: using nodemon (works on both Linux and Windows)
nodemon --ext py --exec coctus -- run --auto-advance --command "python3 sol.py"
```

### Disabling color output

You may disable colors by setting the [NO_COLOR](http://no-color.org/) environment variable to any non-empty value:

```console
$ NO_COLOR=1 coctus show
```


## Installation

The program has only been tested on Linux and Windows. Other platforms may or may not work!

### (Option 1) Download executable

Download the latest binary from [releases](https://github.com/Andriamanitra/coctus/releases) and extract it somewhere on your `$PATH`.

### (Option 2) Build from source:

Building from source requires a relatively recent (1.73+ should work) version of the Rust toolchain.
```
$ git clone https://github.com/Andriamanitra/coctus
$ cargo install --path=.
```
