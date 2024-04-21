import glob
import random
import sys
from subprocess import run
from os.path import expanduser, basename, splitext

CLASH_DIR = expanduser("~/.local/share/clash/clashes")
CLASH_EXE = "target/release/clash"


def check_stubgen(*, clash_ids: list[str], langs_to_check: dict[str, list[str]]) -> dict:
    """
    Checks that stubgen generates valid code for all given clash_ids.
    Returns the number of errors encountered.
    langs_to_check should be a dict where keys are language names for the
    `clash generate-stub` and values are commands that read the generated
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
        run([CLASH_EXE, "next", cid], capture_output=True)

        for lang, check_cmd in langs_to_check.items():
            res = results[lang]
            run_stubgen = run([CLASH_EXE, "generate-stub", lang], capture_output=True)
            if run_stubgen.returncode != 0:
                stderr = run_stubgen.stderr.decode("utf-8")
                if "provides no input stub generator" in stderr:
                    res["n_skipped"] += 1
                else:
                    res["n_checked"] += 1
                    res["n_errors"] += 1
                    print(f"\nStub generator for {lang} returned non-zero for clash {cid}")
                    print(SEPARATOR)
                    print(run_stubgen.stderr.decode("utf-8"))
                    print(SEPARATOR)
                    print()
            else:
                run_check = run(check_cmd, input=run_stubgen.stdout, capture_output=True)
                res["n_checked"] += 1
                if run_check.returncode != 0:
                    res["n_errors"] += 1
                    print(f"\nError with {lang} stub for clash {cid}")
                    print(f"https://www.codingame.com/contribute/view/{cid}")
                    print(SEPARATOR)
                    print(run_check.stderr.decode("utf-8"))
                    print(SEPARATOR)
                    print(run_stubgen.stdout.decode("utf-8"))
                    print(SEPARATOR)
                    print()

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
        "c": ["gcc", "-fsyntax-only", "-x", "c", "-"],
        "cpp": ["gcc", "-fsyntax-only", "-x", "c++", "-"],
        "rust": ["rustc", "--emit=metadata", "-"],
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
