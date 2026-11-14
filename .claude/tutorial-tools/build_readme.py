"""Build the tutorial pages from their templates, then verify them against the source files.

Usage: python build_readme.py <template> <tutorial_dir>

<template> (README.tmpl.md) becomes <tutorial_dir>/README.md, the contents page. Next to it:

- every `chapters/*.tmpl.md` becomes <tutorial_dir>/chapters/<name>.md. Chapters are read in
  file-name order (hence the `NN-` prefixes). Each must have exactly one `# ` title; the build
  inserts a navigation line under that title and again at the end of the page:
  `[Previous: <title>](NN-prev.md) · [Contents](../README.md) · [Next: <title>](NN-next.md)`
  (no Previous on the first chapter, no Next on the last). Templates contain no nav lines.
- every `concepts/*.tmpl.md` becomes <tutorial_dir>/concepts/<name>.md.

Template directives (each on a line of its own):

`@@file <path>` is replaced by a fenced block holding that file.
`@@excerpt <path> ||| <start> ||| <end> [||| <extra>]` is replaced by a fenced block holding
part of a file: from the first line containing <start> to the next line containing <end>,
plus <extra> more lines (for closing braces). Excerpts therefore cannot go stale.
The fence language follows the file extension (.rs, .sql, .toml).
`@@concept-links` is replaced by one `- [<title>](concepts/<name>.md)` line per concept page
(not the concepts index), using each page's `# ` title, in the order the concepts index
template (`concepts/README.tmpl.md`) links them; any page it does not link comes last, in
file-name order.
Include paths are relative to <tutorial_dir> on every page; links and images are relative
to the page itself.

Verification, on every generated page (exit 1 on any failure):
- every ```rust block appears verbatim in a .rs file under <tutorial_dir>/code
  (blocks fenced as ```rust,ignore are illustrative and are skipped);
- every image exists;
- every link between pages resolves, including #anchors, and every other local link exists;
- concepts/README.md links every concept page;
- README.md links every chapter, in file-name order, and its section sub-bullets are
  indented enough to nest under their numbered chapter (4 spaces under `10. `).

Every page is written to disk before it is verified, so a build that fails has already
overwritten its outputs; fix the template or code and build again.
"""
import os
import pathlib
import re
import sys

FENCE_LANGUAGES = {".rs": "rust", ".sql": "sql", ".toml": "toml"}
EXTERNAL = ("http:", "https:", "mailto:")
TEMPLATE_SUFFIX = ".tmpl.md"


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


def slug(heading):
    """GitHub's anchor for a heading: lowercase, drop punctuation, spaces to hyphens."""
    return re.sub(r"[^\w\- ]", "", heading.lower().replace("`", "")).replace(" ", "-")


def render(template_path, tutorial_dir, directives=None):
    """Return (page text, files included, excerpts included) for one template.

    `directives` maps a whole-line directive (such as "@@concept-links") to its lines.
    """
    directives = directives or {}
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
        elif line.strip() in directives:
            out_lines.extend(directives[line.strip()])
        else:
            out_lines.append(line)
    return "\n".join(out_lines), included, excerpts


def prose_lines(text):
    """The lines of a page that are not inside a fenced code block."""
    inside = False
    for line in text.split("\n"):
        if line.lstrip().startswith("```"):
            inside = not inside
        elif not inside:
            yield line


def anchors_of(text):
    """The anchors GitHub generates for a page; repeated headings get -1, -2, ..."""
    seen = {}
    anchors = set()
    for line in prose_lines(text):
        heading = re.match(r"#{1,6} (.+)$", line)
        if heading:
            base = slug(heading.group(1))
            count = seen.get(base, 0)
            anchors.add(base if count == 0 else f"{base}-{count}")
            seen[base] = count + 1
    return anchors


def links_of(page, text):
    """Yield (raw link, target path, fragment) for each non-image, non-external link."""
    prose = re.sub(r"`[^`]*`", "", "\n".join(prose_lines(text)))
    for raw in re.findall(r"(?<!!)\[[^\]]*\]\(([^)\s]+)\)", prose):
        if raw.startswith(EXTERNAL):
            continue
        path_part, _, fragment = raw.partition("#")
        target = pathlib.Path(os.path.normpath(page.parent / path_part)) if path_part else page
        yield raw, target, fragment


def check_links(pages, exists_on_disk=lambda path: False):
    """One message per broken link. `pages` maps output path to page text.

    A .md target must be one of `pages`; any other target must satisfy `exists_on_disk`.
    A #fragment on a generated page must match one of its headings.
    """
    anchors = {path: anchors_of(text) for path, text in pages.items()}
    broken = []
    for page, text in pages.items():
        for raw, target, fragment in links_of(page, text):
            if target.suffix == ".md":
                if target not in pages:
                    broken.append(f"{page}: link to missing page: {raw}")
                elif fragment and fragment not in anchors[target]:
                    broken.append(f"{page}: link to missing heading: {raw}")
            elif not exists_on_disk(target):
                broken.append(f"{page}: link to missing file: {raw}")
    return broken


def missing_images(page, text):
    """Image paths are relative to the page that references them."""
    images = re.findall(r"!\[[^\]]*\]\(([^)]+)\)", "\n".join(prose_lines(text)))
    return [image for image in images if not (page.parent / image).exists()]


def check_rust_blocks(text, sources):
    """Return (blocks not found in any source, whole-file block count, block count)."""
    blocks = re.findall(r"^```rust\n(.*?)\n```$", text, flags=re.S | re.M)
    mismatches = []
    whole_files = 0
    for block in blocks:
        homes = [path for path, source in sources.items() if block in source]
        if not homes:
            mismatches.append(block)
        elif any(sources[path].rstrip("\n") == block for path in homes):
            whole_files += 1
    return mismatches, whole_files, len(blocks)


def concept_link_order(index_text, names):
    """`names` (concept page names without `.md`) in the order `index_text` first links them.

    Only links to a sibling page (`name.md`, optionally with `#anchor`) count. Names the index
    does not link follow in their given order.
    """
    linked = re.findall(r"\]\(([\w-]+)\.md(?:#[^)]*)?\)", index_text)
    ordered = list(dict.fromkeys(name for name in linked if name in names))
    return ordered + [name for name in names if name not in ordered]


def unlisted_concept_pages(pages, concept_index):
    """Messages for concept pages that concepts/README.md does not link to."""
    concept_pages = [p for p in pages if p.parent == concept_index.parent and p != concept_index]
    if not concept_pages:
        return []
    if concept_index not in pages:
        return [f"{concept_index}: missing, but concept pages exist"]
    linked = {target for _, target, _ in links_of(concept_index, pages[concept_index])}
    return [f"{concept_index}: no link to {p.name}" for p in concept_pages if p not in linked]


def title_line(text):
    """Line index of the page's one `# ` title; ValueError unless there is exactly one.

    `# ` lines inside fenced code blocks (TOML or shell comments) are not headings.
    """
    inside = False
    found = []
    for index, line in enumerate(text.split("\n")):
        if line.lstrip().startswith("```"):
            inside = not inside
        elif not inside and line.startswith("# "):
            found.append(index)
    if len(found) != 1:
        raise ValueError(f"expected exactly one '# ' title, found {len(found)}")
    return found[0]


def title_of(text):
    return text.split("\n")[title_line(text)][len("# "):].strip()


def add_navigation(chapters, texts):
    """Return {chapter: text} with a nav line under each title and at the end of the page.

    `chapters` is the reading order. The nav names the neighbours by their `# ` title and
    links the contents page (../README.md). ValueError, naming the chapter, if a chapter
    does not have exactly one title.
    """
    titles = {}
    for chapter in chapters:
        try:
            titles[chapter] = title_of(texts[chapter])
        except ValueError as error:
            raise ValueError(f"{chapter}: {error}") from None
    pages = {}
    for position, chapter in enumerate(chapters):
        parts = []
        if position > 0:
            previous = chapters[position - 1]
            parts.append(f"[Previous: {titles[previous]}]({previous.name})")
        parts.append("[Contents](../README.md)")
        if position + 1 < len(chapters):
            following = chapters[position + 1]
            parts.append(f"[Next: {titles[following]}]({following.name})")
        nav = " · ".join(parts)
        lines = texts[chapter].rstrip("\n").split("\n")
        heading = title_line(texts[chapter])
        lines[heading + 1:heading + 1] = ["", nav]
        if lines[heading + 3:heading + 4] != [""]:
            lines.insert(heading + 3, "")
        pages[chapter] = "\n".join(lines) + "\n\n---\n\n" + nav + "\n"
    return pages


def contents_problems(readme, text, chapters):
    """Messages if the contents page misses a chapter or lists them out of reading order.

    A chapter's place in the contents is where it is first linked (with or without #anchor).
    """
    wanted = set(chapters)
    mentioned = []
    for _, target, _ in links_of(readme, text):
        if target in wanted and target not in mentioned:
            mentioned.append(target)
    problems = [f"{readme}: no link to chapter {c.name}" for c in chapters if c not in mentioned]
    expected = [c for c in chapters if c in mentioned]
    if mentioned != expected:
        problems.append(f"{readme}: chapters listed out of file-name order: "
                        f"{[c.name for c in mentioned]}")
    return problems


def list_indent_problems(page, text):
    """Messages for indented bullets that would not nest under the ordered item above them.

    In CommonMark a child must start at its parent's content column: `9. ` needs 3 spaces,
    `10. ` needs 4. Fewer spaces turn the child into a separate top-level list.
    """
    problems = []
    content_column = None
    for line in prose_lines(text):
        ordered = re.match(r"( *)(\d+\. )", line)
        bullet = re.match(r"( +)[-*+] ", line)
        if ordered and not line.startswith(" "):
            content_column = len(ordered.group(1)) + len(ordered.group(2))
        elif bullet and content_column and len(bullet.group(1)) < content_column:
            problems.append(f"{page}: sub-bullet needs {content_column} spaces to nest under "
                            f"its numbered item: {line.strip()}")
        elif line.startswith("#"):
            content_column = None
    return problems


def outputs_for(template_dir, folder, tutorial_dir):
    """{output page: template} for every <template_dir>/<folder>/*.tmpl.md, in file-name order."""
    return {
        tutorial_dir / folder / (template.name[: -len(TEMPLATE_SUFFIX)] + ".md"): template
        for template in sorted((template_dir / folder).glob("*" + TEMPLATE_SUFFIX))
    }


def main(template_path, tutorial_dir):
    template_path, tutorial_dir = pathlib.Path(template_path), pathlib.Path(tutorial_dir)
    readme = tutorial_dir / "README.md"
    concept_index = tutorial_dir / "concepts" / "README.md"
    chapter_templates = outputs_for(template_path.parent, "chapters", tutorial_dir)
    concept_templates = outputs_for(template_path.parent, "concepts", tutorial_dir)
    templates = {readme: template_path, **chapter_templates, **concept_templates}

    concept_pages = {
        output.stem: (output, template)
        for output, template in concept_templates.items()
        if output != concept_index
    }
    index_template = concept_templates.get(concept_index)
    index_text = read(index_template) if index_template else ""
    concept_links = [
        f"- [{title_of(read(template))}]({output.relative_to(tutorial_dir).as_posix()})"
        for output, template in (
            concept_pages[name] for name in concept_link_order(index_text, list(concept_pages)))
    ]

    pages = {}
    counts = {}
    for output, template in templates.items():
        pages[output], included, excerpts = render(
            template, tutorial_dir, {"@@concept-links": concept_links})
        counts[output] = (included, excerpts)

    chapters = list(chapter_templates)
    problems = []
    try:
        pages.update(add_navigation(chapters, pages))
    except ValueError as error:
        problems.append(str(error))

    for output, text in pages.items():
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text, encoding="utf-8", newline="\n")

    sources = {
        path: read(path)
        for path in (tutorial_dir / "code").rglob("*.rs")
        if "target" not in path.parts
    }

    failed = False
    for output, text in pages.items():
        mismatches, whole_files, total = check_rust_blocks(text, sources)
        images = missing_images(output, text)
        included, excerpts = counts[output]
        print(f"{output.relative_to(tutorial_dir).as_posix()}: {len(text.splitlines())} lines, "
              f"{included} files and {excerpts} generated excerpts included")
        print(f"  rust blocks checked: {total} ({whole_files} whole files, {total - whole_files} excerpts)")
        print(f"  blocks not found in any source file: {len(mismatches)}")
        for block in mismatches:
            print("--- MISMATCH ---\n" + block[:300])
        print(f"  missing images: {images}")
        failed = failed or bool(mismatches or images)

    broken = check_links(pages, lambda path: path.exists())
    broken += unlisted_concept_pages(pages, concept_index)
    print(f"broken links: {broken}")
    problems += contents_problems(readme, pages[readme], chapters)
    problems += list_indent_problems(readme, pages[readme])
    print(f"chapter problems: {problems}")
    sys.exit(1 if (failed or broken or problems) else 0)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
