from decimal import Decimal
from typing import Any, Iterable, overload

class Money:
    def __init__(self, amount: Money | Decimal | float | int | str = 0) -> None: ...
    @property
    def amount(self) -> Decimal: ...
    def round(self, n: int) -> Money: ...
    def round_up(self, n: int) -> Money: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...
    def __add__(self, other: Money) -> Money: ...
    def __radd__(self, other: Money) -> Money: ...
    def __sub__(self, other: Money) -> Money: ...
    def __rsub__(self, other: Money) -> Money: ...
    def __mul__(self, other: Decimal | float | int) -> Money: ...
    def __rmul__(self, other: Decimal | float | int) -> Money: ...
    @overload
    def __truediv__(self, other: Money) -> Decimal: ...
    @overload
    def __truediv__(self, other: Decimal | float | int) -> Money: ...
    def __truediv__(self, other: Money | Decimal | float | int) -> Money | Decimal: ...
    @overload
    def __rtruediv__(self, other: Money) -> Decimal: ...
    @overload
    def __rtruediv__(self, other: Decimal | float | int) -> Money: ...
    def __rtruediv__(self, other: Money | Decimal | float | int) -> Money | Decimal: ...
    def __neg__(self) -> Money: ...
    def __abs__(self) -> Money: ...
    def __bool__(self) -> bool: ...
    def __eq__(self, other: Money) -> bool: ...
    def __lt__(self, other: Money) -> bool: ...
    def __le__(self, other: Money) -> bool: ...
    def __gt__(self, other: Money) -> bool: ...
    def __ge__(self, other: Money) -> bool: ...
    def for_json(self) -> str: ...
    @staticmethod
    def validate(value: Any, schema_info: Any) -> Money: ...

def sum_(elems: Iterable[Money | None]) -> Money: ...

class MoneyWithVAT:
    def __init__(
        self,
        net: Money | Decimal | float | int | str | None = None,
        tax: Money | Decimal | float | int | str | None = None,
    ) -> None: ...
    @property
    def net(self) -> Money: ...
    @property
    def tax(self) -> Money: ...
    @property
    def gross(self) -> Money: ...
    @property
    def tax_rate(self) -> Decimal: ...
    @property
    def tax_rate_for_display(self) -> Decimal: ...
    @property
    def is_positive(self) -> bool: ...
    @property
    def is_negative(self) -> bool: ...
    def is_equal_up_to_cents(self, other: MoneyWithVAT) -> bool: ...
    def is_lower_up_to_cents(self, other: MoneyWithVAT) -> bool: ...
    def is_lower_or_equal_up_to_cents(self, other: MoneyWithVAT) -> bool: ...
    def rounded_to_cents(self) -> MoneyWithVAT: ...
    def rounded_to_money_field_precision(self) -> MoneyWithVAT: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...
    def __add__(self, other: MoneyWithVAT) -> MoneyWithVAT: ...
    def __sub__(self, other: MoneyWithVAT) -> MoneyWithVAT: ...
    def __mul__(
        self, other: MoneyWithVATRatio | Decimal | float | int
    ) -> MoneyWithVAT: ...
    def __rmul__(
        self, other: MoneyWithVATRatio | Decimal | float | int
    ) -> MoneyWithVAT: ...
    def __truediv__(self, other: Decimal | float | int) -> MoneyWithVAT: ...
    def __rtruediv__(self, other: Decimal | float | int) -> MoneyWithVAT: ...
    def __neg__(self) -> MoneyWithVAT: ...
    def __abs__(self) -> MoneyWithVAT: ...
    def __bool__(self) -> bool: ...
    def __eq__(self, other: MoneyWithVAT) -> bool: ...
    def __lt__(self, other: MoneyWithVAT) -> bool: ...
    def __le__(self, other: MoneyWithVAT) -> bool: ...
    def __gt__(self, other: MoneyWithVAT) -> bool: ...
    def __ge__(self, other: MoneyWithVAT) -> bool: ...
    @staticmethod
    def max(*args: object) -> MoneyWithVAT: ...
    @staticmethod
    def ratio(dividend: MoneyWithVAT, divisor: MoneyWithVAT) -> MoneyWithVATRatio: ...
    @staticmethod
    def safe_ratio(
        dividend: MoneyWithVAT | None, divisor: MoneyWithVAT | None
    ) -> MoneyWithVATRatio | None: ...
    @staticmethod
    def safe_ratio_decimal(
        dividend: MoneyWithVAT | None,
        divisor: Decimal | None,
    ) -> MoneyWithVAT | None: ...
    @staticmethod
    def fast_sum(iterable: Iterable[MoneyWithVAT | None]) -> MoneyWithVAT: ...
    @staticmethod
    def fast_sum_with_none(
        iterable: Iterable[MoneyWithVAT | None],
    ) -> MoneyWithVAT | None: ...
    def for_json(self) -> dict: ...
    @staticmethod
    def from_json(dict: dict) -> MoneyWithVAT: ...
    @staticmethod
    def validate(value: Any, schema_info: Any) -> MoneyWithVAT: ...
    @staticmethod
    def german_vat_rates() -> list[Decimal]: ...
    @staticmethod
    def known_vat_rates() -> list[Decimal]: ...

class MoneyWithVATRatio:
    def __init__(
        self, net_ratio: Decimal | float | int, gross_ratio: Decimal | float | int
    ) -> None: ...
    @property
    def net_ratio(self) -> Decimal: ...
    @property
    def gross_ratio(self) -> Decimal: ...
    @staticmethod
    def zero() -> MoneyWithVATRatio: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __add__(self, other: MoneyWithVATRatio) -> MoneyWithVATRatio: ...
    def __sub__(self, other: MoneyWithVATRatio) -> MoneyWithVATRatio: ...
    def __mul__(self, other: Decimal | float | int) -> MoneyWithVATRatio: ...
    def __truediv__(self, other: Decimal | float | int) -> MoneyWithVATRatio: ...
    def __neg__(self) -> MoneyWithVATRatio: ...
    def __eq__(self, other: MoneyWithVATRatio) -> bool: ...
    def for_json(self) -> dict: ...
    @staticmethod
    def validate(value: Any, schema_info: Any) -> MoneyWithVATRatio: ...
