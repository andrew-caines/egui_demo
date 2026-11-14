import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))

from build_readme import (  # noqa: E402
    add_navigation, check_links, concept_link_order, contents_problems, list_indent_problems,
    slug)

P = pathlib.Path

CHAPTERS = [P("t/chapters/01-a.md"), P("t/chapters/02-b.md"), P("t/chapters/03-c.md")]


class NavigationTests(unittest.TestCase):
    def setUp(self):
        texts = {
            CHAPTERS[0]: "# Alpha\n\nFirst body.\n",
            # A `# ` line inside a code fence is a comment, not the chapter title.
            CHAPTERS[1]: "```toml\n# not a heading\n```\n\n# Beta\n\n## Part\n\nMiddle body.",
            CHAPTERS[2]: "# Gamma\n\nLast body.",
        }
        self.pages = add_navigation(CHAPTERS, texts)

    def assert_nav(self, path, expected):
        lines = self.pages[path].rstrip("\n").split("\n")
        heading = next(i for i, line in enumerate(lines) if line.startswith("# ") and "not" not in line)
        self.assertEqual(lines[heading + 1], "")
        self.assertEqual(lines[heading + 2], expected)
        self.assertEqual(lines[-1], expected)
        self.assertEqual(sum(line == expected for line in lines), 2)

    def test_first_chapter_has_no_previous(self):
        self.assert_nav(CHAPTERS[0], "[Contents](../README.md) · [Next: Beta](02-b.md)")

    def test_middle_chapter_links_both_neighbours(self):
        self.assert_nav(CHAPTERS[1],
                        "[Previous: Alpha](01-a.md) · [Contents](../README.md) · [Next: Gamma](03-c.md)")

    def test_last_chapter_has_no_next(self):
        self.assert_nav(CHAPTERS[2], "[Previous: Beta](02-b.md) · [Contents](../README.md)")

    def test_body_is_kept(self):
        self.assertIn("## Part\n\nMiddle body.", self.pages[CHAPTERS[1]])
        self.assertIn("```toml\n# not a heading\n```", self.pages[CHAPTERS[1]])

    def test_chapter_without_exactly_one_title_is_an_error(self):
        with self.assertRaises(ValueError):
            add_navigation(CHAPTERS[:1], {CHAPTERS[0]: "## No title"})
        with self.assertRaises(ValueError):
            add_navigation(CHAPTERS[:1], {CHAPTERS[0]: "# One\n\n# Two"})


class ContentsTests(unittest.TestCase):
    README = P("t/README.md")

    def test_complete_contents_in_order_is_fine(self):
        text = ("1. [A](chapters/01-a.md)\n   - [Part](chapters/01-a.md#part)\n"
                "2. [B](chapters/02-b.md)\n3. [C](chapters/03-c.md)")
        self.assertEqual(contents_problems(self.README, text, CHAPTERS), [])

    def test_unlisted_chapter_is_reported(self):
        text = "1. [A](chapters/01-a.md)\n3. [C](chapters/03-c.md)"
        problems = contents_problems(self.README, text, CHAPTERS)
        self.assertEqual(len(problems), 1)
        self.assertIn("02-b.md", problems[0])

    def test_out_of_order_contents_is_reported(self):
        text = "1. [A](chapters/01-a.md)\n2. [C](chapters/03-c.md)\n3. [B](chapters/02-b.md)"
        problems = contents_problems(self.README, text, CHAPTERS)
        self.assertEqual(len(problems), 1)
        self.assertIn("order", problems[0])

    def test_sub_bullet_indented_less_than_its_parent_content_is_reported(self):
        text = ("9. [I](chapters/09-i.md)\n   - [ok](chapters/09-i.md#a)\n"
                "10. [J](chapters/10-j.md)\n   - [short](chapters/10-j.md#b)\n"
                "    - [ok](chapters/10-j.md#c)")
        problems = list_indent_problems(self.README, text)
        self.assertEqual(len(problems), 1)
        self.assertIn("short", problems[0])
        self.assertIn("4 spaces", problems[0])

    def test_section_link_before_chapter_link_does_not_change_order(self):
        # Order is the order of first mention, whether or not the link has a #fragment.
        text = "[B part](chapters/02-b.md#x)\n[A](chapters/01-a.md)\n[C](chapters/03-c.md)"
        self.assertEqual(len(contents_problems(self.README, text, CHAPTERS)), 1)


class BuildReadmeTests(unittest.TestCase):
    def test_slug_matches_github(self):
        self.assertEqual(slug("`CentralPanel` vs `Panel`"), "centralpanel-vs-panel")
        self.assertEqual(slug("3. Core concepts: Hello World"), "3-core-concepts-hello-world")
        self.assertEqual(slug("Pitfall 1: unstable or clashing widget IDs"), "pitfall-1-unstable-or-clashing-widget-ids")

    def test_cross_file_links(self):
        pages = {P("t/README.md"): "## A b\n[x](concepts/p.md#how-it-works) [y](concepts/missing.md)",
                 P("t/concepts/p.md"): "## How it works\n[back](../README.md#a-b) [bad](../README.md#nope) [in](#how-it-works)"}
        broken = check_links(pages)
        self.assertEqual(len(broken), 2)   # missing.md, #nope
        self.assertTrue(any("missing.md" in m for m in broken) and any("nope" in m for m in broken))

    def test_external_links_ignored(self):
        self.assertEqual(check_links({P("t/a.md"): "[d](https://docs.rs/egui)"}), [])

    def test_links_and_headings_in_code_fences_ignored(self):
        text = "## Real\n```\n## Fake heading\n[x](nowhere.md)\n```\n[ok](#real) [bad](#fake-heading)"
        broken = check_links({P("t/a.md"): text})
        self.assertEqual(len(broken), 1)
        self.assertIn("fake-heading", broken[0])


class ConceptLinkOrderTests(unittest.TestCase):
    INDEX = (
        "| Page | What it covers |\n|---|---|\n"
        "| [`Ui`](ui.md) | x |\n| [`Layout`](layout.md) | y |\n| [Back](../chapters/03-a.md) | z |\n"
    )

    def test_pages_follow_the_index_table_not_file_names(self):
        names = ["app-trait", "layout", "ui"]
        # Pages the index links come first, in its order; the rest follow in file-name order.
        self.assertEqual(concept_link_order(self.INDEX, names), ["ui", "layout", "app-trait"])

    def test_links_to_other_files_are_ignored(self):
        self.assertEqual(concept_link_order(self.INDEX, ["ui"]), ["ui"])

    def test_a_page_linked_twice_is_listed_once(self):
        index = "[a](ui.md) [b](layout.md#x) [c](ui.md#y)"
        self.assertEqual(concept_link_order(index, ["layout", "ui"]), ["ui", "layout"])


if __name__ == "__main__":
    unittest.main()
