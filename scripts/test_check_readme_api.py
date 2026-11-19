#!/usr/bin/env python3
"""Regression tests for scripts/check_readme_api.py public function parsing."""

import unittest

import check_readme_api as checker


# A contract whose public functions wrap the opening paren onto a later line.
# `env` is excluded from the argument count, matching the checker's convention.
WRAPPED_SOURCE = """\
impl SorobanLard {
    pub fn init
    (
        env: Env,
        admin: Address,
        secondary_admin: Address,
    ) {
    }

    pub fn update_course
        (
        env: Env,
        course_id: String,
        price: i128,
    ) -> bool {
    }

    pub fn approve_course(env: Env, caller: Address, course_id: String) {
    }

    pub fn is_btc_eth_token(env: Env, token: Address) -> bool {
    }
}
"""


class PublicFunctionsTest(unittest.TestCase):
    def test_wrapped_opening_paren_is_detected(self):
        functions = checker.public_functions(WRAPPED_SOURCE)
        self.assertIn("init", functions)
        self.assertIn("update_course", functions)

    def test_wrapped_opening_paren_keeps_correct_arity(self):
        functions = checker.public_functions(WRAPPED_SOURCE)
        self.assertEqual(functions["init"], 2)
        self.assertEqual(functions["update_course"], 2)

    def test_single_line_signatures_still_parse(self):
        functions = checker.public_functions(WRAPPED_SOURCE)
        self.assertEqual(functions["approve_course"], 2)
        self.assertEqual(functions["is_btc_eth_token"], 1)

    def test_all_wrapped_and_inline_functions_counted(self):
        functions = checker.public_functions(WRAPPED_SOURCE)
        self.assertEqual(
            set(functions),
            {"init", "update_course", "approve_course", "is_btc_eth_token"},
        )


if __name__ == "__main__":
    unittest.main()
