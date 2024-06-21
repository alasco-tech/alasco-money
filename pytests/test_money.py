# This set of tests is copied from the py-moneyed library
# https://github.com/py-moneyed/py-moneyed/blob/d734ffa7ebd28040cc3f3fcb376876751989e54a/moneyed/test_moneyed_classes.py

from decimal import Decimal

import pytest
from alasco_money import Money


class CustomDecimal(Decimal):
    """Test class to ensure Decimal.__str__ is not
    used in calculations.
    """

    def __str__(self):
        return "error"


def test_init():
    one_million_dollars = Money(Decimal("1000000"))
    assert one_million_dollars.amount == Decimal("1000000")


def test_init_float():
    one_million_dollars = Money(1000000.0)
    assert one_million_dollars.amount == Decimal("1000000")


def test_repr():
    assert repr(Money(Decimal("1000000"))) == "Money('1000000')"
    assert repr(Money(Decimal("2.000"))) == "Money('2.000')"
    m_1 = Money(Decimal("2.00"))
    m_2 = Money(Decimal("2.01"))
    assert repr(m_1) != repr(m_2)


def test_eval_from_repr():
    m = Money("1000")
    assert m == eval(repr(m))  # noqa: PGH001


def test_str():
    assert isinstance(str(Money(Decimal("1000000"))), str)


def test_hash():
    assert Money(Decimal("1000000")) in {Money(Decimal("1000000"))}


def test_add():
    assert Money(Decimal("1000000")) + Money(Decimal("1000000")) == Money(
        amount="2000000"
    )


def test_add_non_money():
    with pytest.raises(TypeError):
        Money(1000) + "123"


def test_sub():
    zeroed_test = Money(Decimal("1000000")) - Money(Decimal("1000000"))
    assert zeroed_test == Money(0)


def test_sub_non_money():
    with pytest.raises(TypeError):
        Money(1000) - "123"


def test_rsub_non_money():
    assert 0 - Money(1) == Money(-1)
    with pytest.raises(TypeError):
        assert "123" - Money(3) == Money(-2)


def test_mul():
    x = Money(111.33)
    assert 3 * x == Money(333.99)
    assert Money(333.99) == 3 * x


def test_mul_bad():
    with pytest.raises(TypeError):
        Money(Decimal("1000000")) * Money(Decimal("1000000"))


def test_div():
    x = Money(50)
    y = Money(2)

    assert x / y == Decimal(25)


def test_div_by_non_Money():
    x = Money(50)
    y = 2
    assert x / y == Money(25)


def test_rdiv_by_non_Money():
    x = 50
    y = Money(2)
    assert x / y == Money(25)


def test_ne():
    x = Money(1)
    assert Money(Decimal("1000000")) != x


def test_equality_to_other_types():
    x = Money(0)
    assert x != None  # noqa: E711
    assert x != {}


def test_not_equal_to_decimal_types():
    assert Money(Decimal("1000000")) != Decimal("1000000")


def test_lt():
    x = Money(1)
    assert x < Money(Decimal("1000000"))


def test_gt():
    x = Money(1)
    assert Money(Decimal("1000000")) > x


def test_abs():
    abs_money = Money(1)
    x = Money(-1)
    assert abs(x) == abs_money
    y = Money(1)
    assert abs(y) == abs_money


def test_sum():
    assert sum([Money(1), Money(2)]) == Money(3)


def test_round():
    x = Money("1234.33569")
    assert x.round(-4) == Money("0")
    assert x.round(-3) == Money("1000")
    assert x.round(-2) == Money("1200")
    assert x.round(-1) == Money("1230")
    assert x.round(0) == Money("1234")
    assert x.round(None) == Money("1234")
    assert x.round(1) == Money("1234.3")
    assert x.round(2) == Money("1234.34")
    assert x.round(3) == Money("1234.336")
    assert x.round(4) == Money("1234.3357")


def test_round_up_down():
    x = Money("2.5")
    assert x.round(0) == Money(2)
    x = Money("3.5")
    assert x.round(0) == Money(4)

    x = Money("2.5")
    assert x.round_up(0) == Money(3)
    x = Money("3.5")
    assert x.round_up(0) == Money(4)


def test_bool():
    assert bool(Money(1))
    assert bool(Money("0.0000000000000000000000000001"))
    assert not bool(Money(0))


def test_decimal_doesnt_use_str_when_multiplying():
    m = Money("531")
    a = CustomDecimal("53.313")
    result = m * a
    assert result == Money("28309.203")


def test_decimal_doesnt_use_str_when_dividing():
    m = Money("15.60")
    a = CustomDecimal("3.2")
    result = m / a
    assert result == Money("4.875")
