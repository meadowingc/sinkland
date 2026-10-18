"""Offline checks for layout-independent packaged paper validation."""

import importlib.util
from pathlib import Path
import unittest


spec = importlib.util.spec_from_file_location(
    "smoke_papers", Path(__file__).resolve().parents[1] / "scripts" / "smoke-papers.py"
)
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)

PAPER = "/papers/p/v2.cs.0000000c.3.000000000000000f"
TABLE = '<div><table class="observations"><tr><td>1.00</td></tr></table></div>'


def paper(experiment_levels=("h3", "h3"), table_counts=(2, 2)):
    sections = [
        ("intro", "h2", "<p>Introduction.</p>"),
        ("methods", "h2", "<p>Methods.</p>"),
        ("definitions", "h3", "<p>Definitions without a figure.</p>"),
        ("results", "h2", "<p>Results.</p>"),
    ]
    for index, (level, count) in enumerate(zip(experiment_levels, table_counts)):
        sections.append((
            f"experiment-{index}", level,
            TABLE * count + f'<figure><img src="{PAPER}/figures/{index}">'
            f'<figcaption>Figure {index + 1}.</figcaption></figure>',
        ))
    sections.extend([
        ("comparison", "h2", "<p>Cross-setting comparison.</p>"),
        ("limitations", "h2", "<p>Limitations.</p>"),
        ("conclusion", "h2", "<p>Conclusion.</p>"),
    ])
    contents = '<nav class="paper-contents"><ol>' + "".join(
        f'<li><a href="#section-{anchor}">{anchor}</a></li>'
        for anchor, level, _ in sections if level == "h2"
    ) + "</ol></nav>"
    body = "".join(
        f'<section class="paper-section{" paper-subsection" if level == "h3" else ""}" '
        f'id="section-{anchor}"><{level}>{anchor}</{level}>{content}</section>'
        for anchor, level, content in sections
    )
    return ('<article class="research-paper">'
            '<section class="paper-abstract"><h2>Abstract</h2></section>'
            + contents + body
            + '<section><h2>References</h2></section></article>')


class PaperSmokeTests(unittest.TestCase):
    def validate(self, html):
        smoke.check_structure(smoke.Page(html), PAPER)

    def test_accepts_experiments_at_either_level_and_prose_only_subsections(self):
        for levels in (("h3", "h3"), ("h2", "h2"), ("h2", "h3"), ("h3", "h2")):
            with self.subTest(levels=levels):
                html = paper(levels)
                if levels[0] == levels[1]:
                    self.assertNotEqual(html.count("<h3>"), 2)
                self.validate(html)

    def test_checks_each_experiment_not_just_the_global_table_total(self):
        html = paper(table_counts=(3, 1))
        self.assertEqual(smoke.Page(html).tables, 4)
        with self.assertRaisesRegex(AssertionError, "two tables per experiment"):
            self.validate(html)

    def test_rejects_tables_in_prose_only_sections(self):
        with self.assertRaisesRegex(AssertionError, "two tables per experiment"):
            self.validate(paper().replace("<p>Methods.</p>", TABLE))

    def test_rejects_missing_or_duplicate_section_anchors(self):
        for html in (
            paper().replace('id="section-intro"', ""),
            paper().replace('id="section-methods"', 'id="section-intro"'),
        ):
            with self.subTest(html=html):
                with self.assertRaisesRegex(AssertionError, "section anchor"):
                    self.validate(html)

    def test_rejects_missing_or_extra_section_headings(self):
        for heading in ("", "<h2>intro</h2><h3>Unexpected</h3>"):
            with self.subTest(heading=heading):
                with self.assertRaisesRegex(AssertionError, "one h2 or h3 heading"):
                    self.validate(paper().replace("<h2>intro</h2>", heading))

    def test_contents_must_cover_exactly_the_top_level_sections(self):
        html = paper(("h2", "h3"))
        for changed in (
            html.replace('<li><a href="#section-experiment-0">experiment-0</a></li>', ""),
            html.replace("#section-intro", "#section-definitions"),
            html.replace("#section-intro", "#section-missing"),
            html.replace("#section-intro", "#section-methods"),
        ):
            with self.subTest(html=changed):
                with self.assertRaisesRegex(AssertionError, "contents do not match"):
                    self.validate(changed)

    def test_rejects_misnumbered_or_foreign_figure_links(self):
        for source in (f"{PAPER}/figures/0", f"{PAPER}/figures/99", "/papers/p/other/figures/1"):
            with self.subTest(source=source):
                with self.assertRaisesRegex(AssertionError, "figure links"):
                    self.validate(paper().replace(f"{PAPER}/figures/1", source))

    def test_rejects_figures_and_tables_outside_paper_sections(self):
        html = paper()
        with self.assertRaisesRegex(AssertionError, "outside a paper section"):
            self.validate(html + f'<img src="{PAPER}/figures/2">')
        with self.assertRaisesRegex(AssertionError, "unexpected table count"):
            self.validate(html + TABLE)

    def test_rejects_missing_figures_and_multiple_figures_in_one_section(self):
        html = paper()
        image = f'<img src="{PAPER}/figures/0">'
        with self.assertRaisesRegex(AssertionError, "expected 2-10 figures"):
            self.validate(html.replace(image, ""))
        with self.assertRaisesRegex(AssertionError, "multiple experiment figures"):
            self.validate(html.replace(image, image + f'<img src="{PAPER}/figures/2">'))


if __name__ == "__main__":
    unittest.main()
