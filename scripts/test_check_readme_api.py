import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from check_readme_api import split_top_level, mentions


class SplitTopLevelTests(unittest.TestCase):
    def test_nested_generic_commas(self):
        parts = split_top_level("a, Vec<Address>, b")
        self.assertEqual(len(parts), 3)
        self.assertEqual([p.strip() for p in parts], ["a", "Vec<Address>", "b"])

    def test_arrow_does_not_inflate_count(self):
        parts = split_top_level("a -> b, c")
        self.assertEqual(len(parts), 2)
        self.assertEqual([p.strip() for p in parts], ["a -> b", "c"])

    def test_stray_closer_does_not_go_negative(self):
        parts = split_top_level("a > b, c")
        self.assertEqual(len(parts), 2)

    def test_nested_parens(self):
        parts = split_top_level("f(x, y), z")
        self.assertEqual(len(parts), 2)


class MentionsTests(unittest.TestCase):
    def test_mention_with_nested_generics_and_arrow(self):
        readme = (
            "Some docs.\n"
            "`transfer(from: Address, to: Vec<Address>, amount: i128)`\n"
            "More docs.\n"
        )
        result = mentions(readme)
        self.assertIn("transfer", result)
        self.assertEqual(len(result["transfer"]), 3)

    def test_mention_with_arrow(self):
        readme = "`convert(a -> b, c)`\n"
        result = mentions(readme)
        self.assertIn("convert", result)
        self.assertEqual(len(result["convert"]), 2)


if __name__ == "__main__":
    unittest.main()