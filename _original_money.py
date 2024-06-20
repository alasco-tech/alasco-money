import decimal as _decimal
import re as _re
from collections.abc import Callable, Iterable, Sequence
from typing import (
    Any,
    NoReturn,
    TypedDict,
    cast,
    overload,
)

import alasco.lib.currency as _currency
import babel.core as _babel_core
import babel.numbers as _babel_numbers
import django.db.models as _django_models
import django.forms as _django_forms
import django_stubs_ext as _django_stubs
import pydantic as _pydantic
import pydantic.json_schema as _json_schema
import pydantic_core.core_schema as _core_schema
import selina.common.field as _selina_fields
import selina.cost_processing.forms.legacy_widgets as _selina_forms
import structlog as _structlog
from django.db.models import functions as _dj_model_functions

logger = _structlog.get_logger(__name__)


class DecimalArgs(TypedDict):
    max_digits: int
    decimal_places: int


DECIMAL_MONEY_ARGS: DecimalArgs = {"max_digits": 28, "decimal_places": 12}
DECIMAL_ARGS: DecimalArgs = {"max_digits": 32, "decimal_places": 16}


def get_current_locale() -> str:
    from django.conf import settings as _settings
    from django.utils import translation as _translation

    locale = _babel_numbers.LC_NUMERIC

    if _settings.configured:  # called within Django context -> load locale
        # get_language can return None starting from Django 1.8
        language = _translation.get_language() or _settings.LANGUAGE_CODE
        locale = _translation.to_locale(language)

    return locale


def get_currency_symbol() -> str:
    currency = _currency.get_currency()
    return (
        _babel_numbers.get_currency_symbol(
            locale=get_current_locale(), currency=currency.value
        )
        if currency
        else ""
    )


class MoneyFormatter:
    def __init__(
        self,
        value: "Money",
        override_locale: str | None = None,
        show_decimals: bool = True,
    ) -> None:
        self._show_decimals = show_decimals
        self._locale = override_locale or get_current_locale()
        self._babel_locale = _babel_core.Locale.parse(self._locale)
        self._value = value

    @property
    def _is_patching_currency_pattern_necessary(self) -> bool:
        # We patch the currency pattern for DKK, HUF, NOK, RON, CHF when the locale is en
        # to make sure, we have a space between the currency symbol and the value.
        return self._value.currency in (
            _currency.Currencies.DKK.value,
            _currency.Currencies.HUF.value,
            _currency.Currencies.NOK.value,
            _currency.Currencies.RON.value,
            _currency.Currencies.CHF.value,
        ) and self._locale.startswith("en")

    def _patch_currency_format_pattern(self) -> str | None:
        format_pattern = self._babel_locale.currency_formats["standard"].pattern

        regex = _re.compile(r"^(¤{1,3})(#.*)")
        match = regex.match(format_pattern)

        if match:
            first_group, second_group = match.groups()
            new_pattern = f"{first_group}\xa0{second_group}"
            return new_pattern

        logger.warning(
            "alasco_lib.money.unable_to_patch_currency_format",
            pattern=format_pattern,
        )
        return None

    def _get_format_pattern(self) -> str | None:
        format_pattern: str | None = None
        if self._is_patching_currency_pattern_necessary:
            patched_pattern = self._patch_currency_format_pattern()
            if patched_pattern:
                format_pattern = patched_pattern

        if not self._show_decimals:
            existing_format_pattern = (
                format_pattern
                or self._babel_locale.currency_formats["standard"].pattern
            )
            format_pattern = _re.sub(r"\.0*", "", existing_format_pattern)

        return format_pattern

    def format(self) -> str:
        kwargs: dict[str, Any] = {
            "number": self._value.amount,
            "currency": self._value.currency,
            "locale": self._babel_locale,
        }

        if not self._show_decimals:
            kwargs["currency_digits"] = False

        pattern = self._get_format_pattern()
        if pattern:
            kwargs["format"] = pattern

        return _babel_numbers.format_currency(**kwargs)


def format_money(
    money: "Money", override_locale: str | None = None, show_decimals: bool = True
) -> str:
    """
    See https://github.com/py-moneyed/py-moneyed/blob/master/moneyed/l10n.py
    See https://babel.pocoo.org/en/latest/api/numbers.html

    override_locale may be used to force a particular locale for testing.
    """
    return MoneyFormatter(
        value=money, override_locale=override_locale, show_decimals=show_decimals
    ).format()


def get_money_format() -> str:
    locale = get_current_locale()
    babel_locale = _babel_core.Locale.parse(locale)
    pattern = babel_locale.currency_formats["standard"].pattern

    return pattern


class Money:
    __slots__ = ("_amount",)

    def __init__(self, amount: "Money | _decimal.Decimal | str | float" = 0) -> None:
        if isinstance(amount, Money):
            self._amount = amount.amount
        elif isinstance(amount, float):
            self._amount = _decimal.Decimal(str(amount))
        else:
            self._amount = _decimal.Decimal(amount)

    @property
    def amount(self) -> _decimal.Decimal:
        return self._amount

    @property
    def currency(self) -> str:
        account_currency = _currency.get_currency()

        if account_currency is None:
            currency_code = _currency.DEFAULT_CURRENCY.value
            logger.warning(
                "alasco_lib.money.currency_not_set",
                fallback_currency=currency_code,
                stack_info=True,
            )
            return currency_code

        return account_currency.value

    def __str__(self) -> str:
        return format_money(self)

    def __repr__(self) -> str:
        return f"Money('{self.amount}')"

    def __add__(self, other: "Money") -> "Money":
        if not isinstance(other, Money):
            return NotImplemented

        return Money(self.amount + other.amount)

    def __radd__(self, other: "int | Money") -> "Money":
        if isinstance(other, int):
            # sum() over Money instances with no `start` argument will attempt
            # to do 0 + Money so we should support this (but only this) case.
            if other == 0:
                return self

            return NotImplemented

        return other.__add__(self)

    def __sub__(self, other: "Money") -> "Money":
        return self.__add__(-other)

    def __rsub__(self, other: "Money") -> "Money":
        return (-self).__radd__(other)

    def __mul__(self, other: _decimal.Decimal | float) -> "Money":
        if isinstance(other, Money):
            return NotImplemented

        if isinstance(other, float):
            return Money(self.amount * _decimal.Decimal(str(other)))

        return Money(self.amount * other)

    @overload
    def __truediv__(self, other: _decimal.Decimal | float) -> "Money": ...

    @overload
    def __truediv__(self, other: "Money") -> _decimal.Decimal: ...

    def __truediv__(
        self, other: "_decimal.Decimal | float | Money"
    ) -> "Money | _decimal.Decimal":
        if isinstance(other, Money):
            return self.amount / other.amount

        if isinstance(other, float):
            other = _decimal.Decimal(str(other))

        return Money(self.amount / other)

    def __rtruediv__(self, other: Any) -> Any:
        return NotImplemented

    def __bool__(self) -> bool:
        return bool(self.amount)

    def __pos__(self) -> "Money":
        return Money(self.amount)

    def __neg__(self) -> "Money":
        return Money(-self.amount)

    def __abs__(self) -> "Money":
        return Money(abs(self.amount))

    __rmul__ = __mul__

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Money) and (self.amount == other.amount)

    def __ne__(self, other: object) -> bool:
        return not self.__eq__(other)

    def __lt__(self, other: "Money") -> bool:
        return self.amount < other.amount

    def __gt__(self, other: "Money") -> bool:
        return self.amount > other.amount

    def __le__(self, other: "Money") -> bool:
        return self < other or self == other

    def __ge__(self, other: "Money") -> bool:
        return self > other or self == other

    def __hash__(self) -> int:
        return hash(self.amount)

    def round(self, n: int = 0) -> "Money":
        return Money(amount=round(self.amount, n))

    @_pydantic.model_serializer()
    def for_json(self) -> str | None:
        if self is None:
            return None
        # rounding as in DecimalField.to_representation
        context = _decimal.getcontext().copy()
        context.prec = DECIMAL_MONEY_ARGS["max_digits"]
        rounded_amount = self.amount.quantize(
            _decimal.Decimal(".1") ** DECIMAL_MONEY_ARGS["decimal_places"],
            context=context,
        )
        return f"{rounded_amount:f}"  # this fixes display of 0.0 as 0E-12

    def for_public_api(self) -> dict | None:
        if self is None:
            return None
        return {"amount": self.for_json(), "currency": self.currency}

    @classmethod
    def __get_pydantic_json_schema__(
        cls,
        core_schema: _core_schema.JsonSchema,
        handler: _pydantic.GetJsonSchemaHandler,
    ) -> _json_schema.JsonSchemaValue:
        return {"example": "123.123456789012", "type": "string"}

    @classmethod
    def validate(cls, value: Any, _: _core_schema.ValidationInfo) -> "Money":
        return _pydantic_validate_money(value)

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source: type[Any], handler: Callable[[Any], _core_schema.CoreSchema]
    ) -> _core_schema.CoreSchema:
        return _core_schema.with_info_plain_validator_function(
            cls.validate,
            serialization=_core_schema.plain_serializer_function_ser_schema(
                cls.for_json, when_used="json"
            ),
        )

    def deconstruct(self) -> tuple[str, list[_decimal.Decimal], dict]:
        """Needed by the Django migration system. Returns a 3-tuple of
        class import path, positional arguments, and keyword arguments."""

        return (
            f"{self.__class__.__module__}.{self.__class__.__name__}",
            [self._amount],
            {},
        )


def _pydantic_validate_money(values: Any) -> Money:
    if isinstance(values, Money):
        return values

    return Money(_pydantic.TypeAdapter(_decimal.Decimal).validate_python(values))


class InvalidJsonStructure(Exception):
    pass


class MoneyWithVATRatio:
    __slots__ = ("_net_ratio", "_gross_ratio")

    def __init__(
        self, net_ratio: _decimal.Decimal, gross_ratio: _decimal.Decimal
    ) -> None:
        self._net_ratio = net_ratio
        self._gross_ratio = gross_ratio

    def __str__(self) -> str:
        return f"MoneyWithVATRatio({self.net_ratio}, {self.gross_ratio})"

    __repr__ = __str__

    @property
    def net_ratio(self) -> _decimal.Decimal:
        return self._net_ratio

    @property
    def gross_ratio(self) -> _decimal.Decimal:
        return self._gross_ratio

    @classmethod
    def from_money_with_vat(
        cls, dividend: "MoneyWithVAT", divisor: "MoneyWithVAT"
    ) -> "MoneyWithVATRatio":
        return cls(
            net_ratio=dividend.net / divisor.net,
            gross_ratio=dividend.gross / divisor.gross,
        )

    @classmethod
    def zero(cls) -> "MoneyWithVATRatio":
        return cls(net_ratio=_decimal.Decimal(0), gross_ratio=_decimal.Decimal(0))

    @overload
    def __mul__(self, other: int | _decimal.Decimal) -> "MoneyWithVATRatio": ...

    @overload
    def __mul__(self, other: "MoneyWithVAT") -> "MoneyWithVAT": ...

    def __mul__(
        self, other: "MoneyWithVAT | int | _decimal.Decimal"
    ) -> "MoneyWithVAT | MoneyWithVATRatio":
        if isinstance(other, int | _decimal.Decimal):
            return MoneyWithVATRatio(
                net_ratio=self.net_ratio * other, gross_ratio=self.gross_ratio * other
            )

        if isinstance(other, MoneyWithVAT):
            amount = self.net_ratio * other.net
            return MoneyWithVAT(net=amount, tax=other.gross * self.gross_ratio - amount)

        return NotImplemented

    def __add__(self, other: "MoneyWithVATRatio") -> "MoneyWithVATRatio":
        if isinstance(other, MoneyWithVATRatio):
            return MoneyWithVATRatio(
                net_ratio=self.net_ratio + other.net_ratio,
                gross_ratio=self.gross_ratio + other.gross_ratio,
            )

        raise TypeError(
            f"__add__: `other` argument must be of type {type(self)}, got {type(other)} instead"
        )

    def __neg__(self) -> "MoneyWithVATRatio":
        return MoneyWithVATRatio(
            net_ratio=-self.net_ratio,
            gross_ratio=-self.gross_ratio,
        )

    def __sub__(self, other: "MoneyWithVATRatio") -> "MoneyWithVATRatio":
        return self.__add__(-other)

    def __truediv__(self, other: _decimal.Decimal | int) -> "MoneyWithVATRatio":
        if isinstance(other, int | _decimal.Decimal):
            return self * (_decimal.Decimal(1) / other)

        raise TypeError(
            f"__truediv__: `other` argument must be of type `int` or `_decimal.Decimal`, got {type(other)} instead"
        )

    def __eq__(self, other: object) -> bool:
        if isinstance(other, MoneyWithVATRatio):
            return (
                self.net_ratio == other.net_ratio
                and self.gross_ratio == other.gross_ratio
            )
        return False

    @_pydantic.model_serializer()
    def for_json(self) -> dict[str, str] | None:
        if self is None:
            return None
        return {"net_ratio": str(self.net_ratio), "gross_ratio": str(self.gross_ratio)}

    @classmethod
    def __get_pydantic_json_schema__(
        cls,
        core_schema: _core_schema.JsonSchema,
        handler: _pydantic.GetJsonSchemaHandler,
    ) -> _json_schema.JsonSchemaValue:
        return {
            "properties": {
                "net_ratio": {
                    "title": "Net ratio",
                    "type": "string",
                    "example": "0.23",
                },
                "gross_ratio": {
                    "title": "Gross ratio",
                    "type": "string",
                    "example": "0.23",
                },
            },
            "type": "object",
        }

    @classmethod
    def validate(
        cls, value: Any, _: _core_schema.ValidationInfo
    ) -> "MoneyWithVATRatio":
        return _pydantic_validate_money_vat_ratio(value)

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source: type[Any], handler: Callable[[Any], _core_schema.CoreSchema]
    ) -> _core_schema.CoreSchema:
        return _core_schema.with_info_plain_validator_function(
            cls.validate,
            serialization=_core_schema.plain_serializer_function_ser_schema(
                cls.for_json, when_used="json"
            ),
        )


def _pydantic_validate_money_vat_ratio(values: Any) -> MoneyWithVATRatio:
    if isinstance(values, MoneyWithVATRatio):
        return values

    assert isinstance(values, dict), "Either have to pass a dict or MoneyWithVATRatio"
    assert set(values) == {"net_ratio", "gross_ratio"}
    return MoneyWithVATRatio(
        **{
            key: _pydantic.TypeAdapter(_decimal.Decimal).validate_python(val)
            for key, val in values.items()
        }
    )


class MoneyWithVAT:
    __slots__ = ("_net", "_tax")

    # Known VAT rates in countries
    # Germany (0.19, 0.16, 0.07, 0.05)
    # Austria (0.20, 0.13, 0.10),
    # Denmark (0.25)
    GERMAN_VAT_RATES = tuple(
        map(
            _decimal.Decimal,
            ("0.19", "0.16", "0.07", "0.05", "0"),
        )
    )

    _AUSTRIAN_VAT_RATES = tuple(
        map(
            _decimal.Decimal,
            ("0.20", "0.13", "0.10", "0"),
        )
    )

    _DENMARK_VAT_RATES = tuple(map(_decimal.Decimal, ("0.25",)))

    _KNOWN_VAT_RATES = tuple(
        set(GERMAN_VAT_RATES + _AUSTRIAN_VAT_RATES + _DENMARK_VAT_RATES)
    )

    def __init__(
        self,
        net: _decimal.Decimal | str | float | Money | None = 0,
        tax: _decimal.Decimal | str | float | Money | None = 0,
    ) -> None:
        if not isinstance(net, Money):
            net = Money(net or 0)

        if not isinstance(tax, Money):
            tax = Money(tax or 0)

        self._net = net
        self._tax = tax

    @property
    def net(self) -> Money:
        return self._net

    @property
    def tax(self) -> Money:
        return self._tax

    @property
    def gross(self) -> Money:
        return self.net + self.tax

    @classmethod
    def fast_sum(cls, operands: Iterable["MoneyWithVAT | None"]) -> "MoneyWithVAT":
        net, tax = _decimal.Decimal(0), _decimal.Decimal(0)

        for operand in operands:
            if not operand:
                continue

            net += operand.net.amount
            tax += operand.tax.amount

        return cls(net=net, tax=tax)

    @classmethod
    def fast_sum_with_none(
        cls, operands: Iterable["MoneyWithVAT | None"]
    ) -> "MoneyWithVAT | None":
        """
        This is a variation of fast_sum, that returns None if only None values are given.
        """
        result = cls.fast_sum(operands)
        if not result and all(operand is None for operand in operands):
            return None
        return result

    @classmethod
    def from_json(cls, money_dict: dict[str, Any]) -> "MoneyWithVAT":
        try:
            amount_with_vat = money_dict["amount_with_vat"]
            net_amount = amount_with_vat["net"]["amount"]
            gross_amount = amount_with_vat["gross"]["amount"]
        except (TypeError, KeyError) as exc:
            raise InvalidJsonStructure(
                "Can not create MoneyWithVAT instance from given dictionary!", exc
            ) from exc

        try:
            net = _decimal.Decimal(str(net_amount))
            tax = _decimal.Decimal(str(gross_amount)) - net
        except _decimal.InvalidOperation as exc:
            raise InvalidJsonStructure(
                "InvalidOperation: Can not map value to decimal", exc
            ) from exc

        return cls(net=net, tax=tax)

    @property
    def tax_rate(self) -> _decimal.Decimal:
        if self._net.amount == 0:
            return _decimal.Decimal(0)

        return self._tax / self._net

    @property
    def tax_rate_for_display(self) -> _decimal.Decimal:
        """
        Tax rate as decimal which is rounded to nearest known "real" VAT ratio
        if applicable (19.01 ==> 19.00; but not 23 ==> 19)

        ATTENTION: Don't use the result of this for calculations!
        """
        tax_rate = self.tax_rate
        if tax_rate in self._KNOWN_VAT_RATES:
            return tax_rate

        for rate in self._KNOWN_VAT_RATES:
            vat = rate * self._net
            vat_diff = abs(vat - self._tax)
            if vat_diff.amount < _decimal.Decimal("0.05"):
                return rate

        return tax_rate

    def __neg__(self) -> "MoneyWithVAT":
        return MoneyWithVAT(net=-1 * self._net, tax=-1 * self._tax)

    def __lt__(self, other: "MoneyWithVAT") -> bool:
        return self.gross < other.gross

    def __gt__(self, other: "MoneyWithVAT") -> bool:
        return self.gross > other.gross

    def __add__(self, other: "MoneyWithVAT") -> "MoneyWithVAT":
        return MoneyWithVAT(net=self.net + other.net, tax=self.tax + other.tax)

    def __radd__(self, other: "int | MoneyWithVAT") -> "MoneyWithVAT":
        if isinstance(other, int):
            # sum() over MoneyWithVAT instances with no `start` argument will attempt
            # to do 0 + MoneyWithVAT so we should support this (but only this) case.
            if other == 0:
                return self

            return NotImplemented

        return other.__add__(self)

    def __sub__(self, other: "MoneyWithVAT") -> "MoneyWithVAT":
        return self.__add__(-other)

    def __mul__(self, other: int | _decimal.Decimal) -> "MoneyWithVAT":
        """Multiply Money with single 'Number', i.e. amount & tax will be multiplied by `other`"""
        if isinstance(other, MoneyWithVATRatio):
            return other * self

        return MoneyWithVAT(net=self.net * other, tax=self.tax * other)

    def __eq__(self, other: object) -> bool:
        if isinstance(other, MoneyWithVAT):
            return self.net == other.net and self.tax == other.tax
        return False

    def __bool__(self) -> bool:
        return bool(self.net) or bool(self.tax)

    def __truediv__(self, other: _decimal.Decimal | int) -> "MoneyWithVAT":
        """Divide Money with single 'Number', i.e. amount & tax will be divided by `other`"""
        return self * (_decimal.Decimal(1) / other)

    def __hash__(self) -> int:
        return hash((self.net, self.tax))

    def __repr__(self) -> str:
        return f"<MoneyVAT: net={self.net.amount} tax={self.tax.amount}>"

    @classmethod
    def ratio(
        cls, dividend: "MoneyWithVAT", divisor: "MoneyWithVAT"
    ) -> MoneyWithVATRatio:
        """
        return the exact net/gross ratio of the provided amounts.
        Raises if the divisor is 0 or too if the resulting ratio is too big for the DecimalContext
        """
        if not all(isinstance(elem, MoneyWithVAT) for elem in (dividend, divisor)):
            raise TypeError(
                f"All arguments must be of type {MoneyWithVAT}, got {type(dividend)} & {type(divisor)} instead"
            )

        return MoneyWithVATRatio.from_money_with_vat(dividend, divisor)

    @classmethod
    def safe_ratio(
        cls, dividend: "MoneyWithVAT | None", divisor: "MoneyWithVAT | None"
    ) -> MoneyWithVATRatio | None:
        """
        return a net/gross ratio of the provided amounts rounded to cents.
        return None on bad input, i.e. when the divisor is 0 or too small or either value is None.

        We round here because division by small amounts is unsafe: divisors like 0.0000000000000003
        easily lead to large ratio numbers that can exceed our DecimalContext later in the code.
        """

        try:
            return MoneyWithVATRatio.from_money_with_vat(
                dividend.rounded_to_cents(),  # type: ignore[union-attr]
                divisor.rounded_to_cents(),  # type: ignore[union-attr]
            )
        except (
            _decimal.InvalidOperation,
            _decimal.DivisionByZero,
            TypeError,
            AttributeError,
        ):
            return None

    @classmethod
    def safe_ratio_decimal(
        cls, dividend: "MoneyWithVAT | None", divisor: _decimal.Decimal | None
    ) -> "MoneyWithVAT | None":
        if dividend is not None and divisor:
            return dividend / divisor
        return None

    @property
    def is_negative(self) -> bool:
        return self.gross.amount < 0

    @property
    def is_positive(self) -> bool:
        return self.gross.amount > 0

    def rounded_to_cents(self) -> "MoneyWithVAT":
        """
        Use with caution - only intended for displaying money or before comparing exact amounts with user input.

        Effects of rounding money to cents include:
            (a) the .tax_rate is no longer accurate (e.g. 0.1882 instead of 0.19)
            (b) the .tax can differ from the rounded tax of the original values
                (e.g. with net=4.444 & tax=2.222 the displayed tax is 2.22 EUR, but after .round_to_cents
                 on net=4.44 & tax=2.23 so as to keep gross stable in 6.67 EUR)
            (c) Comparing them later to their exact counterparts returns False
            (d) Ratios formed from rounded amounts no longer add to 100%
        """

        rounded_net = self.net.round(2)
        return MoneyWithVAT(rounded_net, self.gross.round(2) - rounded_net)

    def rounded_to_money_field_precision(self) -> "MoneyWithVAT":
        """When storing Money, values are implicitly rounded to the field precision,
        which is lower than the precision of our DecimalContext.

        This method returns an equivalently rounded value for comparison.
        """
        money_field_precision = DECIMAL_MONEY_ARGS["decimal_places"]

        return MoneyWithVAT(
            self.net.round(money_field_precision), self.tax.round(money_field_precision)
        )

    def is_equal_up_to_cents(self, other: "MoneyWithVAT") -> bool:
        return self.gross.round(2) == other.gross.round(2)

    def is_lower_up_to_cents(self, other: "MoneyWithVAT") -> bool:
        return self.gross.round(2) < other.gross.round(2)

    def is_lower_or_equal_up_to_cents(self, other: "MoneyWithVAT") -> bool:
        return self.is_equal_up_to_cents(other) or self.is_lower_up_to_cents(other)

    @overload
    @classmethod
    def max(
        cls, __arg1: "MoneyWithVAT", __arg2: "MoneyWithVAT", *_args: "MoneyWithVAT"
    ) -> "MoneyWithVAT": ...

    @overload
    @classmethod
    def max(cls, __args: Sequence["MoneyWithVAT"]) -> "MoneyWithVAT": ...

    @classmethod
    def max(cls, *args: "MoneyWithVAT | Sequence[MoneyWithVAT]") -> "MoneyWithVAT":
        moneys = cast(Sequence["MoneyWithVAT"], args[0] if len(args) == 1 else args)
        max_amount = max(money.net for money in moneys)

        return cls(
            net=max_amount, tax=max(money.gross for money in moneys) - max_amount
        )

    __rsub__ = __sub__
    __rmul__ = __mul__
    __rtruediv__ = __truediv__

    @_pydantic.model_serializer()
    def for_json(self) -> dict[str, str | None] | None:
        if self is None:
            return None
        return {"net": self.net.for_json(), "tax": self.tax.for_json()}

    def for_public_api(self) -> dict | None:
        if self is None:
            return None
        return dict(self.for_json(), currency=self.net.currency)  # type: ignore[arg-type]

    @classmethod
    def __get_pydantic_json_schema__(
        cls,
        core_schema: _core_schema.JsonSchema,
        handler: _pydantic.GetJsonSchemaHandler,
    ) -> _json_schema.JsonSchemaValue:
        return {
            "properties": {
                "net": {
                    "example": "123.123456789012",
                    "title": "Net amount",
                    "type": "string",
                },
                "tax": {
                    "example": "123.123456789012",
                    "title": "Tax amount",
                    "type": "string",
                },
            },
            "type": "object",
        }

    @classmethod
    def validate(cls, value: Any, _: _core_schema.ValidationInfo) -> "MoneyWithVAT":
        return _pydantic_validate_money_vat(value)

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source: type[Any], handler: Callable[[Any], _core_schema.CoreSchema]
    ) -> _core_schema.CoreSchema:
        return _core_schema.with_info_plain_validator_function(
            cls.validate,
            serialization=_core_schema.plain_serializer_function_ser_schema(
                cls.for_json, when_used="json"
            ),
        )


def _pydantic_validate_money_vat(values: Any) -> MoneyWithVAT:
    if isinstance(values, MoneyWithVAT):
        return values

    assert isinstance(values, dict), "Either have to pass a dict or MoneyWithVAT"
    assert set(values) == {"net", "tax"}, "You have to specify both net and tax"
    return MoneyWithVAT(
        **{
            key: _pydantic.TypeAdapter(Money).validate_python(val)
            for key, val in values.items()
        }
    )


class _MoneyFieldProxy:
    """A property descriptor which
    (a) casts Decimal field values into Money on field access
    (b) ensures that `field.to_python()` is called on field assignment"""

    def __init__(self, field: _django_models.Field) -> None:
        self.field = field

    def __get__(self, obj: object, type: type | None = None) -> Any:
        if obj is None:
            return self

        if obj.__dict__[self.field.name] is None:
            return None

        if not isinstance(obj.__dict__[self.field.name], Money):
            # Copied from djmoney: store the Money on first access, so
            # the conversion Decimal -> Money does only happens once.
            obj.__dict__[self.field.name] = Money(obj.__dict__[self.field.name])

        return obj.__dict__[self.field.name]

    def __set__(self, obj: object, value: Any) -> None:
        obj.__dict__[self.field.name] = self.field.to_python(value)


class MoneyField(_django_models.DecimalField):
    """This is basically a DecimalField that
    (a) unpacks Money values to Decimals on assignment and
    (b) adds the _MoneyFieldProxy descriptor which casts Decimals into Money values when accessing fields

    This is roughly how djmoney's MoneyField worked as well"""

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        kwargs.setdefault("decimal_places", DECIMAL_MONEY_ARGS["decimal_places"])
        kwargs.setdefault("max_digits", DECIMAL_MONEY_ARGS["max_digits"])
        super().__init__(*args, **kwargs)

    def get_db_prep_save(self, value: Any, connection: Any) -> Any:
        if isinstance(value, Money):
            value = value.amount
        return super().get_db_prep_save(value, connection)

    def to_python(self, value: Any) -> Any:
        if isinstance(value, Money):
            value = value.amount
        if isinstance(value, float):
            value = str(value)
        return super().to_python(value)

    def contribute_to_class(
        self, cls: type[_django_models.Model], name: str, private_only: bool = False
    ) -> None:
        super().contribute_to_class(cls, name, private_only=private_only)
        setattr(cls, name, _MoneyFieldProxy(self))

    def formfield(self, **kwargs: Any) -> _django_forms.Field:  # type: ignore[override]
        from django.db.models.fields import NOT_PROVIDED

        defaults = {
            "form_class": _selina_forms.LegacyMoneyField,
            "decimal_places": self.decimal_places,
            "currency_choices": _currency.Currencies.choices(),
        }
        defaults.update(kwargs)

        if self.default not in (None, NOT_PROVIDED):
            defaults["default_amount"] = self.default
        return super().formfield(**defaults)  # type: ignore[arg-type]


class Round2Digits(_dj_model_functions.Round):
    lookup_name = "round2"

    def __init__(self, expression: Any, **extra: Any) -> None:
        super().__init__(expression, precision=2, **extra)


MoneyField.register_lookup(Round2Digits)
_django_models.DecimalField.register_lookup(Round2Digits)


class MoneyWithVATField(_selina_fields.NonDatabaseFieldBase):
    description = "A field that combines net and tax fields values into one Field"

    def __init__(
        self,
        net_field: str,
        tax_field: str,
        verbose_name: _django_stubs.StrOrPromise | None = None,
    ) -> None:
        super().__init__()
        self.net_field, self.tax_field = net_field, tax_field
        self.verbose_name = verbose_name
        self.default = MoneyWithVAT()

    def __str__(self) -> str:
        return f"MoneyWithVATField({self.net_field}, {self.tax_field})"

    @overload
    def __get__(
        self, instance: None, cls: type | None = None
    ) -> "MoneyWithVATField": ...

    @overload
    def __get__(self, instance: object, cls: type | None = None) -> MoneyWithVAT: ...

    def __get__(
        self, instance: object | None, cls: type | None = None
    ) -> "MoneyWithVATField | MoneyWithVAT":
        if instance is None:
            return self

        net_amount = getattr(instance, self.net_field)
        tax_amount = getattr(instance, self.tax_field)

        return MoneyWithVAT(net_amount, tax_amount)

    def __set__(self, instance: object, value: "MoneyWithVAT | None") -> None:
        net_amount, tax_amount = _decimal.Decimal(0), _decimal.Decimal(0)

        if value is not None:
            net_amount, tax_amount = value.net.amount, value.tax.amount

        setattr(instance, self.net_field, net_amount)
        setattr(instance, self.tax_field, tax_amount)

    def formfield(self, **kwargs: Any) -> NoReturn:
        raise NotImplementedError("I guess you should not use this field in a form!")


class MoneyWithVATFieldNullable(_selina_fields.NonDatabaseFieldBase):
    description = (
        "A field that combines net and tax fields values into one Field "
        "-- and returns None if both net_field/tax_field are None"
    )

    def __init__(
        self,
        net_field: str,
        tax_field: str,
        verbose_name: _django_stubs.StrOrPromise | None = None,
    ) -> None:
        super().__init__()
        self.net_field, self.tax_field = net_field, tax_field
        self.verbose_name = verbose_name
        self.default = None

    def __str__(self) -> str:
        return f"MoneyWithVATFieldNullable({self.net_field}, {self.tax_field})"

    @overload
    def __get__(
        self, instance: None, cls: type | None = None
    ) -> "MoneyWithVATFieldNullable": ...

    @overload
    def __get__(self, instance: object, cls: type | None = None) -> MoneyWithVAT: ...

    def __get__(
        self, instance: object, cls: type | None = None
    ) -> "MoneyWithVATFieldNullable | MoneyWithVAT | None":
        if instance is None:
            return self

        net_amount = getattr(instance, self.net_field)
        tax_amount = getattr(instance, self.tax_field)

        if net_amount is None and tax_amount is None:
            return None

        return MoneyWithVAT(net_amount, tax_amount)

    def __set__(self, instance: object, value: "MoneyWithVAT | None") -> None:
        net_amount, tax_amount = None, None

        if value is not None:
            net_amount, tax_amount = value.net.amount, value.tax.amount

        setattr(instance, self.net_field, net_amount)
        setattr(instance, self.tax_field, tax_amount)

    def formfield(self, **kwargs: Any) -> NoReturn:
        raise NotImplementedError("I guess you should not use this field in a form!")


def sum_(elems: Iterable[Money]) -> Money:
    """
    Sums Money elements while ignoring None values. Is ok with empty lists/iterables.
    """
    return sum((elem for elem in elems if elem is not None), Money())
