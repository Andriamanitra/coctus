# clash

Command line tool that allows you to play [Clash of Code](https://www.codingame.com/multiplayer/clashofcode) style problems locally on your computer.


## Usage

These commands should be enough to get you started:

```
# Download a couple of clashes by id (last part of the URL on the contribution page)
clash fetch 52980368bdbd05abdd789a04173b57b0fdea 682102420fbce0fce95e0ee56095ea2b9924

# View information such as the id of the current problem and the directory where clashes are being stored
clash status

# Move on to next (random) problem
clash next

# Show the current problem
clash show

# Select a specific problem
clash next 52980368bdbd05abdd789a04173b57b0fdea

# Show a specific problem
clash show 52980368bdbd05abdd789a04173b57b0fdea

# Run Python script "sol.py" against the tests of the current problem
clash run --command "python3 sol.py"

# Compile a C program "sol.c" and run it against the tests of the current problem
clash run --build-command "gcc -o sol sol.c" --command "./sol"
```

Use `clash help` and `clash <SUBCOMMAND> --help` to show all available options.

### Auto-refreshing using `entr` or `nodemon`

We recommend using a program such as [entr](https://github.com/eradman/entr) or [nodemon](https://www.npmjs.com/package/nodemon) to refresh the statement when the current clash changes and/or automatically run tests when a file is saved.
Combining `clash` with one of these tools allows you to never need to leave your text editor while clashing.

#### Example 1: Automatically show the new problem when it changes:

This example uses `entr` (or `nodemon`) to detect changes to the file that keeps track of the current clash and run the command `clash show`.
The `-c` flag to `entr` clears the terminal before running the command.
Press Ctrl-c to stop.

```
# Option 1: using entr (Linux only)
ls ~/.local/share/clash/current | entr -c clash show

# Option 2: using nodemon (PowerShell on Windows only)
nodemon --watch "$env:APPDATA\Clash CLI\clash\data\current" --exec clash show
```

#### Example 2: Automatically run code when a file is saved

This example uses `entr` (or `nodemon`) to watch over .py files in the current directory and run `python3 sol.py` when any of them are saved to disk.
The `--auto-advance` flag automatically does the equivalent of `clash next` when you pass all the test cases.
Press Ctrl-c to stop.

```
# Option 1: using entr (Linux only)
ls *.py | entr clash run --auto-advance --command "python3 sol.py"

# Option 2: using nodemon (works on both Linux and Windows)
nodemon --ext py --exec clash -- run --auto-advance --command "python3 sol.py"
```

### Disabling color output

You may disable colors by setting the [NO_COLOR](http://no-color.org/) environment variable to any non-empty value:

```console
$ NO_COLOR=1 clash show
```


## Installation

The program has only been tested on Linux and Windows. Other platforms may or may not work!

### (Option 1) Download executable

Download the latest binary from [releases](https://github.com/Andriamanitra/clash/releases) and extract it somewhere on your `$PATH`.

### (Option 2) Build from source:

Building from source requires a relatively recent (1.73+ should work) version of the Rust toolchain.
```
$ git clone https://github.com/Andriamanitra/clash
$ cargo install --path clash
```
