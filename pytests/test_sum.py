import alasco_money as _money
import pytest as _pytest


@_pytest.mark.parametrize(
    "operands, expected",
    [
        ([], _money.Money(0)),
        ([None], _money.Money(0)),
        (
            [_money.Money(100), _money.Money(200), _money.Money(0), None],
            _money.Money(300),
        ),
    ],
)
def test_sum_(operands, expected):
    assert _money.sum_(operands) == expected
