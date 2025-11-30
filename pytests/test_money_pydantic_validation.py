import pydantic as _pydantic
import pytest as _pytest

import alasco_money as _money


def test_have_validators():
    class SomeModel(_pydantic.BaseModel):
        some_money: _money.Money
        even_with_tax: _money.MoneyWithVAT
        ratio: _money.MoneyWithVATRatio


@_pytest.mark.parametrize("data", [123, "123.00000"])
def test_money_validator_ok(data):
    result = _pydantic.TypeAdapter(_money.Money).validate_python(data)
    assert result == _money.Money(123)


@_pytest.mark.parametrize(
    "data",
    [
        {"amount": "a", "currency": "eur"},
        {"currency": "EUR"},
        {"amount": "1"},
    ],
)
def test_money_validator_fail(data):
    with _pytest.raises(_pydantic.ValidationError):
        _pydantic.TypeAdapter(_money.Money).validate_python(data)


def test_money_vat_validator_ok():
    result = _pydantic.TypeAdapter(_money.MoneyWithVAT).validate_python({"net": 100, "tax": 19})
    assert result.net == _money.Money(100)
    assert result.tax == _money.Money(19)


@_pytest.mark.parametrize(
    "data",
    [
        {"tax": 1, "net": None},
        {"net": {"amount": 1, "currency": "eur"}},
    ],
)
def test_money_vat_validator_fail(data):
    with _pytest.raises(_pydantic.ValidationError):
        _pydantic.TypeAdapter(_money.MoneyWithVAT).validate_python(data)


def test_money_vat_ratio_validator_ok():
    result = _pydantic.TypeAdapter(_money.MoneyWithVATRatio).validate_python(
        {"net_ratio": 2, "gross_ratio": 1}
    )
    assert result == _money.MoneyWithVATRatio(net_ratio=2, gross_ratio=1)


@_pytest.mark.parametrize(
    "data",
    [
        {"net_ratio": 1},
        {"gross_ratio": 1},
    ],
)
def test_money_vat_ratio_validator_fail(data):
    with _pytest.raises(_pydantic.ValidationError):
        _pydantic.TypeAdapter(_money.MoneyWithVATRatio).validate_python(data)
