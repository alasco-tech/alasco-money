import decimal as _decimal

import alasco_money as _money
import pytest as _pytest


@_pytest.mark.parametrize(
    "subject",
    [_money.MoneyWithVAT(), _money.MoneyWithVAT(None, None), _money.MoneyWithVAT(0, 0)],
)
def test_default_init(subject):
    assert subject.net.amount == 0
    assert subject.tax.amount == 0
    assert subject.gross.amount == 0


@_pytest.mark.parametrize(
    "subject, expected",
    [
        (
            _money.MoneyWithVAT(_decimal.Decimal("100"), _decimal.Decimal("-100")),
            _money.MoneyWithVAT(_decimal.Decimal("-100"), _decimal.Decimal("100")),
        ),
        (
            _money.MoneyWithVAT(_decimal.Decimal("-100"), _decimal.Decimal("100")),
            _money.MoneyWithVAT(_decimal.Decimal("100"), _decimal.Decimal("-100")),
        ),
    ],
)
def test_neg(subject, expected):
    assert -subject == expected


@_pytest.mark.parametrize(
    "subject, expected",
    [
        (_money.MoneyWithVAT(50), _money.MoneyWithVAT(30)),
        (_money.MoneyWithVAT(50, -10), _money.MoneyWithVAT(60, -30)),
        (_money.MoneyWithVAT(0, 10), _money.MoneyWithVAT(60, -70)),
        (_money.MoneyWithVAT(-10, -10), _money.MoneyWithVAT(-20, -20)),
    ],
)
def test_gt(subject, expected):
    assert subject > expected


@_pytest.mark.parametrize(
    "subject, expected",
    [
        (_money.MoneyWithVAT(30), _money.MoneyWithVAT(50)),
        (_money.MoneyWithVAT(60, -30), _money.MoneyWithVAT(50, -10)),
        (_money.MoneyWithVAT(60, -70), _money.MoneyWithVAT(0, 10)),
        (_money.MoneyWithVAT(-20, -20), _money.MoneyWithVAT(-10, -10)),
    ],
)
def test_lt(subject, expected):
    assert subject < expected


@_pytest.mark.parametrize(
    "first, second, result",
    [
        (
            _money.MoneyWithVAT(1, 1),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(3, 3),
        ),
        (
            _money.MoneyWithVAT(1, -1),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(3, 1),
        ),
        (
            _money.MoneyWithVAT(-1, 1),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(1, 3),
        ),
    ],
)
def test_add(first, second, result):
    assert first + second == result


@_pytest.mark.parametrize(
    "first, second, result",
    [
        (
            _money.MoneyWithVAT(3, 3),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(1, 1),
        ),
        (
            _money.MoneyWithVAT(3, -3),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(1, -5),
        ),
        (
            _money.MoneyWithVAT(-3, 3),
            _money.MoneyWithVAT(2, 2),
            _money.MoneyWithVAT(-5, 1),
        ),
    ],
)
def test_sub(first, second, result):
    assert first - second == result


@_pytest.mark.parametrize("direction", ["forward", "reverse"])
def test_mul_commutative(direction):
    value = 20
    vat = _decimal.Decimal("0.19")
    tax = _decimal.Decimal(20 * vat)
    subject = _money.MoneyWithVAT(value, tax)
    c = subject * 2 if direction == "forward" else 2 * subject
    assert c.net.amount == 2 * value
    assert c.tax.amount == tax * 2


@_pytest.mark.parametrize(
    "subject, expected",
    [
        (_money.MoneyWithVAT(), False),
        (_money.MoneyWithVAT(0), False),
        (_money.MoneyWithVAT(None, 0), False),
        (_money.MoneyWithVAT(0, 0), False),
        (_money.MoneyWithVAT(_decimal.Decimal("0.000000000"), 0), False),
        (_money.MoneyWithVAT(1, 0), True),
        (_money.MoneyWithVAT(0, 1), True),
        (_money.MoneyWithVAT(1, 1), True),
        (_money.MoneyWithVAT(_decimal.Decimal("0.000000001"), 0), True),
    ],
)
def test_bool(subject, expected):
    assert bool(subject) is expected


@_pytest.mark.parametrize(
    "original, divisor, expected",
    [
        (_money.MoneyWithVAT(), 1, _money.MoneyWithVAT()),
        (_money.MoneyWithVAT(100, 19), 1, _money.MoneyWithVAT(100, 19)),
        (_money.MoneyWithVAT(200, 38), 2, _money.MoneyWithVAT(100, 19)),
        (
            _money.MoneyWithVAT(100, 19),
            2,
            _money.MoneyWithVAT(50, _decimal.Decimal("9.5")),
        ),
    ],
)
def test_truediv(original, divisor, expected):
    """
    Test  `MoneyWithVAT` divided by a Number divides both values (amount & tax)
    """
    assert (original / divisor) == expected


@_pytest.mark.parametrize("zero", [0, _decimal.Decimal(0)])
def test_truediv_zero(zero):
    with _pytest.raises(ZeroDivisionError):
        _money.MoneyWithVAT() / zero


def test_truediv_no_money():
    """
    Div method fails if you try to divide MoneyWithVAT by MoneyWithVAT
    """
    with _pytest.raises(TypeError):
        _money.MoneyWithVAT() / _money.MoneyWithVAT()


@_pytest.mark.parametrize(
    "net,tax,rate",
    [
        (100, 19, "0.19"),
        (0, 23, 0),
        (100, 0, 0),
        (-100, -19, "0.19"),
        (300, 20, "0.0666666666666666666666666667"),
    ],
)
def test_tax_rate(net, tax, rate):
    assert _money.MoneyWithVAT(net, tax).tax_rate == _decimal.Decimal(rate)


@_pytest.mark.parametrize(
    "net,tax,rate",
    [
        ("32299.8", "6136.96", "0.19"),
        (100, 19, "0.19"),
        (100, "19.00912", "0.19"),
        (100, "4.991", "0.05"),
        (0, 23, 0),
        (100, 0, 0),
        (-100, -19, "0.19"),
        (300, 30, "0.10"),
    ],
)
@_pytest.mark.xfail
def test_tax_rate_for_display(net, tax, rate):
    assert _money.MoneyWithVAT(net, tax).tax_rate_for_display == _decimal.Decimal(rate)


@_pytest.mark.parametrize(
    "dividend, divisor, expected",
    [
        (_money.MoneyWithVAT(100, 19), _money.MoneyWithVAT(100, 19), (1, 1)),
        (_money.MoneyWithVAT(200, 38), _money.MoneyWithVAT(100, 19), (2, 2)),
        (
            _money.MoneyWithVAT(100, 19),
            _money.MoneyWithVAT(50, _decimal.Decimal("9.5")),
            (2, 2),
        ),
    ],
)
def test_ratio(dividend, divisor, expected):
    ratio = _money.MoneyWithVAT.ratio(dividend, divisor)
    assert ratio.net_ratio == expected[0]
    assert ratio.gross_ratio == expected[1]


@_pytest.mark.parametrize(
    "dividend, divisor", [(_money.MoneyWithVAT(200, 38), _money.MoneyWithVAT(0, 0))]
)
def test_ratio_zero(dividend, divisor):
    with _pytest.raises(ZeroDivisionError):
        _money.MoneyWithVAT.ratio(dividend, divisor)


@_pytest.mark.parametrize(
    "items",
    [
        ((100, 19),),
        ((100, 19), (112, 999), (1, 2), (99999, 0)),
        ((100.0000001, 19), (100.00000001, 999), (1, 2)),
    ],
)
def test_max(items):
    moneys = [_money.MoneyWithVAT(elem[0], elem[1]) for elem in items]
    money_max = _money.MoneyWithVAT.max(moneys)
    net_list = sorted(elem[0] for elem in items)
    gross = sorted(elem[0] + elem[1] for elem in items)

    assert money_max.net.amount == _decimal.Decimal(str(net_list[-1]))
    assert money_max.gross.amount == _decimal.Decimal(str(gross[-1]))


def test_ratio_mul():
    money_a = _money.MoneyWithVAT(100, 19)
    money_b = _money.MoneyWithVAT(200, 14)
    ratio = _money.MoneyWithVAT.ratio(money_a, money_b)
    assert money_a.rounded_to_cents() == (ratio * money_b).rounded_to_cents()
    assert money_a.rounded_to_cents() == (money_b * ratio).rounded_to_cents()

    left = money_a * ratio + money_b * ratio
    right = (money_a + money_b) * ratio
    assert left.net.amount == right.net.amount
    assert left.gross.amount == right.gross.amount


def test_safe_ratio_error_cases():
    money = _money.MoneyWithVAT(1000000, 19)
    money_0 = _money.MoneyWithVAT()
    money_almost_0 = _money.MoneyWithVAT("0.00000000000001", 0)

    assert _money.MoneyWithVAT.safe_ratio(money, money).net_ratio == 1
    assert _money.MoneyWithVAT.safe_ratio(money, money_0) is None
    assert _money.MoneyWithVAT.safe_ratio(money, money_almost_0) is None
    assert _money.MoneyWithVAT.safe_ratio(money, None) is None
    assert _money.MoneyWithVAT.safe_ratio(None, None) is None


@_pytest.mark.parametrize(
    "dividend,divisor,expected",
    [
        (_money.MoneyWithVAT(0), 0, None),
        (_money.MoneyWithVAT(0), None, None),
        (None, 0, None),
        (None, None, None),
        (_money.MoneyWithVAT(0), 1, _money.MoneyWithVAT(0)),
        (_money.MoneyWithVAT(1), 1, _money.MoneyWithVAT(1)),
        (_money.MoneyWithVAT(1), 2, _money.MoneyWithVAT(0.5)),
        (_money.MoneyWithVAT(1), _decimal.Decimal(5), _money.MoneyWithVAT(0.2)),
    ],
)
def test_safe_ratio_decimal(dividend, divisor, expected):
    result = _money.MoneyWithVAT.safe_ratio_decimal(dividend, divisor)

    if expected is not None:
        assert expected.is_equal_up_to_cents(expected)
    else:
        assert result == expected


@_pytest.mark.parametrize(
    "params",
    [
        (_money.MoneyWithVAT(0), _money.MoneyWithVAT("0.001")),
        (_money.MoneyWithVAT("600.001"), _money.MoneyWithVAT("599.9966")),
        (_money.MoneyWithVAT("123.004"), _money.MoneyWithVAT("123.001")),
        (_money.MoneyWithVAT("0.012"), _money.MoneyWithVAT("0.007")),
        (_money.MoneyWithVAT("0.007"), _money.MoneyWithVAT("0.012")),
    ],
)
def test_is_equal_up_to_cents(params):
    m1, m2 = params

    assert m1.is_equal_up_to_cents(m2)


def test_not_is_equal_up_to_cents():
    assert not _money.MoneyWithVAT(0).is_equal_up_to_cents(_money.MoneyWithVAT("0.01"))
    assert not _money.MoneyWithVAT("1.006").is_equal_up_to_cents(
        _money.MoneyWithVAT("1.004")
    )
    assert not _money.MoneyWithVAT("-1.006").is_equal_up_to_cents(
        _money.MoneyWithVAT("-1.004")
    )


@_pytest.mark.parametrize(
    "params",
    [
        (_money.MoneyWithVAT(0), _money.MoneyWithVAT("-0.001")),
        (_money.MoneyWithVAT("600.001"), _money.MoneyWithVAT("599.9966")),
        (_money.MoneyWithVAT("123.004"), _money.MoneyWithVAT("123.001")),
        (_money.MoneyWithVAT("0.002"), _money.MoneyWithVAT("0.007")),
        (_money.MoneyWithVAT("0.012"), _money.MoneyWithVAT("0.007")),
        (_money.MoneyWithVAT("2"), _money.MoneyWithVAT("7")),
    ],
)
def test_is_lower_or_equal_up_to_cents(params):
    m1, m2 = params
    assert m1.is_lower_or_equal_up_to_cents(m2)


def test_not_is_lower_or_equal_up_to_cents():
    assert not _money.MoneyWithVAT(0).is_lower_or_equal_up_to_cents(
        _money.MoneyWithVAT("-0.01")
    )
    assert not _money.MoneyWithVAT("1.006").is_lower_or_equal_up_to_cents(
        _money.MoneyWithVAT("1.004")
    )


@_pytest.mark.parametrize(
    "params",
    [
        (_money.MoneyWithVAT(0), _money.MoneyWithVAT("0.009")),
        (_money.MoneyWithVAT("0.002"), _money.MoneyWithVAT("0.007")),
        (_money.MoneyWithVAT("2"), _money.MoneyWithVAT("7")),
    ],
)
def test_is_lower_up_to_cents(params):
    m1, m2 = params
    assert m1.is_lower_up_to_cents(m2)


def test_not_is_lower_up_to_cents():
    assert not _money.MoneyWithVAT(0).is_lower_up_to_cents(
        _money.MoneyWithVAT("-0.001")
    )
    assert not _money.MoneyWithVAT("0.999").is_lower_up_to_cents(
        _money.MoneyWithVAT("1.004")
    )


@_pytest.mark.parametrize(
    "money_under_test, expected",
    [
        (_money.MoneyWithVAT(0), False),
        (_money.MoneyWithVAT("0.00000000000000000000001"), False),
        (_money.MoneyWithVAT("-0.00000000000000000000001"), True),
        (_money.MoneyWithVAT(_decimal.Decimal(355) / 113), False),
        (_money.MoneyWithVAT(10000000000000000000000), False),
        (_money.MoneyWithVAT(-10000000000000000000000), True),
    ],
)
def test_is_negative(money_under_test, expected):
    assert money_under_test.is_negative is expected


@_pytest.mark.parametrize(
    "money_under_test, expected",
    [
        (_money.MoneyWithVAT(0), False),
        (_money.MoneyWithVAT("0.00000000000000000000001"), True),
        (_money.MoneyWithVAT("-0.00000000000000000000001"), False),
        (_money.MoneyWithVAT(_decimal.Decimal(355) / 113), True),
        (_money.MoneyWithVAT(10000000000000000000000), True),
        (_money.MoneyWithVAT(-10000000000000000000000), False),
    ],
)
def test_is_positive(money_under_test, expected):
    assert money_under_test.is_positive is expected


@_pytest.mark.xfail
def test_from_json_invalid_structure():
    json_input = {
        "amount_with_vat": {
            "grozz": None,
            "net": {"shrug": True},
        }
    }

    with _pytest.raises(_money.InvalidJsonStructure):
        _money.MoneyWithVAT.from_json(json_input)


@_pytest.mark.parametrize("invalid_amount", [None, "not a number"])
@_pytest.mark.xfail
def test_from_json_invalid_amount(invalid_amount):
    json_input = {
        "amount_with_vat": {
            "gross": {"amount": invalid_amount, "currency": "EUR"},
            "net": {"amount": 100, "currency": "USD"},
        },
    }

    with _pytest.raises(_money.InvalidJsonStructure):
        _money.MoneyWithVAT.from_json(json_input)


@_pytest.mark.parametrize("empty_input", [{}, None, []])
@_pytest.mark.xfail
def test_from_json_empty_input(empty_input):
    with _pytest.raises(_money.InvalidJsonStructure):
        _money.MoneyWithVAT.from_json(empty_input)


@_pytest.mark.xfail
def test_from_json_division_by_zero():
    json_input_both_zero = {
        "amount_with_vat": {
            "gross": {"amount": 0, "currency": "EUR"},
            "net": {"amount": 0, "currency": "EUR"},
        },
    }

    json_input_net_zero = {
        "amount_with_vat": {
            "gross": {"amount": 123, "currency": "EUR"},
            "net": {"amount": 0, "currency": "EUR"},
        },
    }

    result_both_zero = _money.MoneyWithVAT.from_json(json_input_both_zero)
    assert result_both_zero.tax.amount == _decimal.Decimal("0")
    assert result_both_zero.tax_rate == _decimal.Decimal("0")

    result_net_zero = _money.MoneyWithVAT.from_json(json_input_net_zero)
    assert result_net_zero.tax.amount == _decimal.Decimal("123")
    assert result_net_zero.tax_rate == _decimal.Decimal("0")


@_pytest.mark.xfail
def test_from_json():
    json_input = {
        "amount_with_vat": {
            "gross": {"amount": 111, "currency": "EUR"},
            "net": {"amount": 100, "currency": "EUR"},
        },
    }
    result = _money.MoneyWithVAT.from_json(json_input)
    assert result.tax.amount == 11
    assert result.tax_rate == _decimal.Decimal("0.11")


@_pytest.mark.parametrize(
    "operands, result",
    [
        ([], _money.MoneyWithVAT(0, 0)),
        (
            [_money.MoneyWithVAT(1, 1), _money.MoneyWithVAT(2, 2)],
            _money.MoneyWithVAT(3, 3),
        ),
        ([_money.MoneyWithVAT(1, -1), None], _money.MoneyWithVAT(1, -1)),
        (
            [_money.MoneyWithVAT(-1, 1), _money.MoneyWithVAT(0, 0)],
            _money.MoneyWithVAT(-1, 1),
        ),
    ],
)
def test_fast_sum(operands, result):
    assert _money.MoneyWithVAT.fast_sum(operands) == result


@_pytest.mark.parametrize(
    "operands, result",
    [
        ([], None),
        (
            [
                _money.MoneyWithVAT(0, 0),
                _money.MoneyWithVAT(0, 0),
                _money.MoneyWithVAT(0, 0),
            ],
            _money.MoneyWithVAT(0, 0),
        ),
        (
            [_money.MoneyWithVAT(1, 1), _money.MoneyWithVAT(2, 2)],
            _money.MoneyWithVAT(3, 3),
        ),
        ([_money.MoneyWithVAT(1, -1), None], _money.MoneyWithVAT(1, -1)),
        (
            [_money.MoneyWithVAT(-1, 1), _money.MoneyWithVAT(0, 0)],
            _money.MoneyWithVAT(-1, 1),
        ),
        ([None, None, None], None),
    ],
)
def test_fast_sum_with_none(operands, result):
    assert _money.MoneyWithVAT.fast_sum_with_none(operands) == result


def _slow_money_vat_sum(operands):
    return sum(
        (operand for operand in operands if operand is not None),
        start=_money.MoneyWithVAT(),
    )


@_pytest.mark.skip("Just for performance testing -- can be flaky")
def test_fast_sum_performance():
    import time

    length = 1000
    lots_of_operands = (
        (40 * [_money.MoneyWithVAT(10, 1)])
        + (2 * [_money.MoneyWithVAT(0, 0)])
        + (2 * [None])
    ) * length
    lots_expected = _money.MoneyWithVAT(400, 40) * length
    some_operands = (8 * [_money.MoneyWithVAT(10, 1)]) + [
        _money.MoneyWithVAT(0, 0),
        None,
    ]

    fast_time = time.time()
    assert _money.MoneyWithVAT.fast_sum(lots_of_operands) == lots_expected
    for _ in range(length):
        _money.MoneyWithVAT.fast_sum(some_operands)
        _money.MoneyWithVAT.fast_sum([])
    fast_time = time.time() - fast_time

    slow_time = time.time()
    assert _slow_money_vat_sum(lots_of_operands) == lots_expected
    for _ in range(length):
        _slow_money_vat_sum(some_operands)
        _slow_money_vat_sum([])
    slow_time = time.time() - slow_time

    assert slow_time > (3 * fast_time)


@_pytest.mark.parametrize(
    "net, tax, expected_net, expected_tax",
    [
        (
            _money.Money("0"),
            _money.Money("0"),
            _money.Money("0"),
            _money.Money("0"),
        ),
        (
            _money.Money("4.444"),
            _money.Money("2.222"),
            _money.Money("4.44"),
            _money.Money("2.23"),
        ),
        (
            _money.Money("25357.9765600"),
            _money.Money("4818.0155464"),
            _money.Money("25357.98"),
            _money.Money("4818.01"),
        ),
    ],
)
def test_rounded_to_cents(net, tax, expected_net, expected_tax):
    value = _money.MoneyWithVAT(net, tax)

    assert value.rounded_to_cents().net == expected_net
    assert value.rounded_to_cents().tax == expected_tax
    assert value.rounded_to_cents().gross == expected_net + expected_tax

    assert (
        value.gross.round(2)
        == value.rounded_to_cents().net + value.rounded_to_cents().tax
    )
    assert value.gross.round(2) == value.rounded_to_cents().gross
