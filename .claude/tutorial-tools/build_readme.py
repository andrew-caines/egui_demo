"""Build tutorial/README.md from the template, then verify it against the source files.

Usage: python build_readme.py <template> <tutorial_dir>

`@@file <path>` lines in the template are replaced by a fenced block holding that file.
`@@excerpt <path> ||| <start> ||| <end> [||| <extra>]` lines are replaced by a fenced block
holding part of a file: from the first line containing <start> to the next line containing
<end>, plus <extra> more lines (for closing braces). Excerpts therefore cannot go stale.
The fence language follows the file extension (.rs, .sql, .toml).

Verification: every ```rust block in the output must appear verbatim in a .rs file under
<tutorial_dir>/code (blocks fenced as ```rust,ignore are illustrative and are skipped).
"""
import pathlib
import re
import sys

template_path, tutorial_dir = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])

FENCE_LANGUAGES = {".rs": "rust", ".sql": "sql", ".toml": "toml"}


def read(path):
    return path.read_text(encoding="utf-8").replace("\r\n", "\n")


def fenced(source, text):
    return ["```" + FENCE_LANGUAGES.get(source.suffix, "text"), text, "```"]


def excerpt(source, start, end, extra):
    lines = read(source).split("\n")
    first = next((i for i, text in enumerate(lines) if start in text), None)
    if first is None:
        sys.exit(f"excerpt start not found in {source}: {start!r}")
    last = next((i for i in range(first, len(lines)) if end in lines[i]), None)
    if last is None:
        sys.exit(f"excerpt end not found in {source}: {end!r}")
    return "\n".join(lines[first:last + 1 + extra])


out_lines = []
included = 0
excerpts = 0
for line in read(template_path).split("\n"):
    if line.startswith("@@file "):
        source = tutorial_dir / line[len("@@file "):].strip()
        out_lines.extend(fenced(source, read(source).rstrip("\n")))
        included += 1
    elif line.startswith("@@excerpt "):
        parts = [part.strip() for part in line[len("@@excerpt "):].split("|||")]
        source = tutorial_dir / parts[0]
        extra = int(parts[3]) if len(parts) > 3 else 0
        out_lines.extend(fenced(source, excerpt(source, parts[1], parts[2], extra)))
        excerpts += 1
    else:
        out_lines.append(line)

readme = "\n".join(out_lines)
(tutorial_dir / "README.md").write_text(readme, encoding="utf-8", newline="\n")

sources = {
    path: read(path)
    for path in (tutorial_dir / "code").rglob("*.rs")
    if "target" not in path.parts
}

blocks = re.findall(r"^```rust\n(.*?)\n```$", readme, flags=re.S | re.M)
mismatches = []
whole_files = 0
for block in blocks:
    homes = [path for path, text in sources.items() if block in text]
    if not homes:
        mismatches.append(block)
    elif any(sources[path].rstrip("\n") == block for path in homes):
        whole_files += 1

images = re.findall(r"!\[[^\]]*\]\(([^)]+)\)", readme)
missing_images = [image for image in images if not (tutorial_dir / image).exists()]

anchors = set()
for heading in re.findall(r"^#{1,6} (.+)$", readme, flags=re.M):
    slug = re.sub(r"[^\w\- ]", "", heading.lower().replace("`", "")).replace(" ", "-")
    anchors.add(slug)
broken_links = [link for link in re.findall(r"\]\(#([^)]+)\)", readme) if link not in anchors]

print(f"README.md: {len(readme.splitlines())} lines, {included} files and {excerpts} generated excerpts included")
print(f"rust blocks checked: {len(blocks)} ({whole_files} whole files, {len(blocks) - whole_files} excerpts)")
print(f"blocks not found in any source file: {len(mismatches)}")
for block in mismatches:
    print("--- MISMATCH ---\n" + block[:300])
print(f"missing images: {missing_images}")
print(f"broken in-page links: {broken_links}")
sys.exit(1 if (mismatches or missing_images or broken_links) else 0)
