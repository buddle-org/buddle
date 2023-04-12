# -*- coding: utf-8 -*-

import re

from .types import convert_field_name

OPERATORS = {
    "#ADD#": "+",
    "#SUB#": "-",
    "#MUL#": "*",
    "#DIV#": "/",
    "#AND#": "&&",
    "#OR#": "||",
    "#LT#": "<",
    "#GT#": ">",
    "#LTE#": "<=",
    "#GTE#": ">=",
    "#EQ#": "==",
    "#NEQ#": "!=",
    "#RSH#": ">>",
    "#LSH#": "<<",
    "#BITAND#": "&",
    "#BITOR#": "|",
}

EXPR_RE_1 = re.compile("(.*)\\((.*)\\) (\\W\\w*\\W|\\W*) \\((.*)\\)")
EXPR_RE_2 = re.compile("(.*)\\((.*) (\\W\\w*\\W|\\W*) (.*)\\)")
EXPR_RE_3 = re.compile("(.*) (\\W\\w*\\W|\\W*) (.*)")


def _contains_any_operator(string):
    return any(k in string or v in string for k, v in OPERATORS.items())


def parse_expression(expr):
    # Sometime expressions use the #___# for operators,
    # sometimes the C versions of them.
    if "(" in expr or ")" in expr or _contains_any_operator(expr):
        no_unary = False
        group_indices = [2, 3, 4]

        # Keep trying regexes here until one matches.
        # This one matches stuff like `!(fizz) #ADD# (buzz)` and `(fizz) + (buzz)`
        result = EXPR_RE_1.search(expr)
        if not result:
            # This one matches stuff like `!(fizz #ADD# buzz)` and `(fizz + buzz)`
            result = EXPR_RE_2.search(expr)
        if not result:
            # This one matches stuff like `fizz #ADD# buzz` and `fizz + buzz`
            result = EXPR_RE_3.search(expr)

            # These variants can't have a unary operator at the start, so the groups are shifted.
            no_unary = True
            group_indices = [1, 2, 3]

        if no_unary:
            unary = None
        else:
            unary = result.group(1)

        # Recursively parse expression.
        expr1 = parse_expression(result.group(group_indices[0]))
        if " " in expr1 and not expr1.startswith("FileVersion"):
            expr1 = f"({expr1})"
        op = result.group(group_indices[1])

        if op.lstrip().rstrip() == "|":
            # | is a weird conditional where only the side whose vars exist
            # is evaluated. In reality the right side is always some weird
            # Bethesda shit.
            return expr1

        # Recursively parse expression.
        expr2 = parse_expression(result.group(group_indices[2]))
        if " " in expr2 and not expr2.startswith("FileVersion"):
            expr2 = f"({expr2})"

        # Replace operator, if necessary.
        op = OPERATORS.get(op, op)

        # Reassemble the expression.
        res = f"{expr1} {op} {expr2}"
        if unary:
            res = f"{unary}({res})"
        return res

    # Any non-numeric is a field name reference.
    if not expr.isnumeric():
        expr = convert_field_name(expr)

    return expr
