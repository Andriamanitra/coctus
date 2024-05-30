import glob
import random
import sys
import tempfile
import os
from subprocess import run
from os.path import expanduser, basename, splitext
from collections.abc import Callable

CLASH_DIR = expanduser("~/.local/share/coctus/clashes")
COCTUS_EXE = "target/release/coctus"

def check_c(src: str) -> bool:
    cmd = ["gcc", "-fsyntax-only", "-x", "c", "-"]
    run_check = run(cmd, input=src, capture_output=True)
    return run_check.returncode == 0

def check_cpp(src: str) -> bool:
    cmd = ["gcc", "-fsyntax-only", "-x", "c++", "-"]
    run_check = run(cmd, input=src, capture_output=True)
    return run_check.returncode == 0

def check_python(src: str) -> bool:
    with tempfile.NamedTemporaryFile() as tmp:
        tmp.write(src)
        tmp.flush()
        cmd = ["mypy", tmp.name]
        run_check = run(cmd, capture_output=True)
        return run_check.returncode == 0

def check_rust(src: str) -> bool:
    cmd = ["rustc", "--emit=metadata", "-"]
    run_check = run(cmd, input=src, capture_output=True)
    garbage = "librust_out.rmeta"
    if os.path.isfile(garbage):
        os.remove(garbage)
    return run_check.returncode == 0


def check_stubgen(*, clash_ids: list[str], langs_to_check: dict[str, Callable]) -> dict:
    """
    Checks that stubgen generates valid code for all given clash_ids.
    Returns the number of errors encountered.
    langs_to_check should be a dict where keys are language names for the
    `coctus generate-stub` and values are commands that read the generated
    code from STDIN and exit non-zero if the code is not valid.

    Example:
    ========
    >>> check_stubgen(
            clash_ids=["682102420fbce0fce95e0ee56095ea2b9924"],
            langs_to_check={"c": ["gcc", "-fsyntax-only", "-x", "c", "-"]}
        )
    """
    SEPARATOR = "=" * 30
    results = {lang: {"n_skipped": 0, "n_checked": 0, "n_errors": 0} for lang in langs_to_check}

    for cid in clash_ids:
        run([COCTUS_EXE, "next", cid], capture_output=True)

        for lang, check_lang_fn in langs_to_check.items():
            res = results[lang]
            run_stubgen = run([COCTUS_EXE, "generate-stub", lang], capture_output=True)
            if run_stubgen.returncode != 0 and "provides no input stub generator" in run_stubgen.stderr.decode():
                res["n_skipped"] += 1
                continue
            res["n_checked"] += 1
            if run_stubgen.returncode != 0:
                stderr = run_stubgen.stderr.decode("utf-8")
                res["n_errors"] += 1
                print(f"\nStub generator for {lang} returned non-zero for clash {cid}")
                print(SEPARATOR)
                print(run_stubgen.stderr.decode("utf-8"))
                print(SEPARATOR)
                print()
            elif not check_lang_fn(run_stubgen.stdout):
                res["n_errors"] += 1
                print(f"\nGenerated bad {lang} stub for clash {cid}")
                print(f"https://www.codingame.com/contribute/view/{cid}")
    return results


def main(n = None):
    clash_ids = []
    for full_clash_path in glob.glob(f"{CLASH_DIR}/*.json"):
        filename = basename(full_clash_path)
        clash_id, _ = splitext(filename)
        clash_ids.append(clash_id)

    if n is not None:
        clash_ids = random.sample(clash_ids, n)

    langs_to_check = {
        "c": check_c,
        "cpp": check_cpp,
        "python": check_python,
        "rust": check_rust,
    }

    lang_results = check_stubgen(clash_ids=clash_ids, langs_to_check=langs_to_check)

    for lang, results in lang_results.items():
        print(f"{lang}: {results}")


if __name__ == "__main__":
    if len(sys.argv) > 1:
        n = int(sys.argv[1])
        main(n)
    else:
        main()
