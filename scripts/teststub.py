import glob
import random
import sys
from subprocess import run
from os.path import expanduser, basename, splitext

CLASH_DIR = expanduser("~/.local/share/clash/clashes")
CLASH_EXE = "target/release/clash"

def check_stubgen(*, clash_ids: list[str], lang: str, check_cmd: list[str]) -> dict:
    """
    Checks that stubgen generates valid code for all given clash_ids.
    Returns the number of errors encountered.
    check_cmd should be a command that reads the code from STDIN and exits non-zero
    if the code is not valid.

    Example:
    ========
    check_stubgen(
        clash_ids=["682102420fbce0fce95e0ee56095ea2b9924"],
        lang="c",
        check_cmd=["gcc", "-fsyntax-only", "-x", "c", "-"]
    )
    """
    SEPARATOR = "=" * 30
    GENERATE_STUB = [CLASH_EXE, "generate-stub", lang]
    n_skipped = 0
    n_checked = 0
    n_errors = 0

    for cid in clash_ids:
        run([CLASH_EXE, "next", cid], capture_output=True)

        run_stubgen = run(GENERATE_STUB, capture_output=True)
        if run_stubgen.returncode != 0:
            stderr = run_stubgen.stderr.decode("utf-8")
            if "Clash provides no input stub generator" in stderr:
                n_skipped += 1
            else:
                n_checked += 1
                n_errors += 1
                print(f"\nStub generator returned non-zero for clash {cid}")
                print(SEPARATOR)
                print(run_stubgen.stderr.decode("utf-8"))
                print(SEPARATOR)
                print()
            continue

        run_check = run(check_cmd, input=run_stubgen.stdout, capture_output=True)
        n_checked += 1
        if run_check.returncode != 0:
            n_errors += 1
            print(f"\nError with stub for clash {cid}")
            print(SEPARATOR)
            print(run_check.stderr.decode("utf-8"))
            print(SEPARATOR)
            print(run_stubgen.stdout.decode("utf-8"))
            print(SEPARATOR)
            print()

    return {
        "stub_language": lang,
        "num_checked": n_checked,
        "num_skipped": n_skipped,
        "num_errors": n_errors
    }


def main(n = None):
    clash_ids = []
    for full_clash_path in glob.glob(f"{CLASH_DIR}/*.json"):
        filename = basename(full_clash_path)
        clash_id, _ = splitext(filename)
        clash_ids.append(clash_id)

    if n is not None:
        clash_ids = random.sample(clash_ids, n)

    GCC_CMD = ["gcc", "-fsyntax-only", "-x", "c", "-"]
    c_results = check_stubgen(clash_ids=clash_ids, lang="c", check_cmd=GCC_CMD)

    GCC_CMD = ["gcc", "-fsyntax-only", "-x", "c++", "-"]
    cpp_results = check_stubgen(clash_ids=clash_ids, lang="cpp", check_cmd=GCC_CMD)

    RUSTC_CMD = ["rustc", "-"]
    rust_results = check_stubgen(clash_ids=clash_ids, lang="rust", check_cmd=RUSTC_CMD)

    print(c_results)
    print(cpp_results)
    print(rust_results)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        n = int(sys.argv[1])
        main(n)
    else:
        main()
