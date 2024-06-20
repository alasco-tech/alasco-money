from decimal import Decimal
from typing import Iterable, Self

class Money:
    def __init__(self, amount: Self | Decimal | float | int | str = 0) -> None: ...
    @property
    def amount(self) -> Decimal: ...
    def round(self, n: int) -> Self: ...
    def round_up(self, n: int) -> Self: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...
    def __add__(self, other: Self) -> Self: ...
    def __radd__(self, other: Self) -> Self: ...
    def __sub__(self, other: Self) -> Self: ...
    def __rsub__(self, other: Self) -> Self: ...
    def __mul__(self, other: Decimal | float | int) -> Self: ...
    def __rmul__(self, other: Decimal | float | int) -> Self: ...
    def __truediv__(self, other: Self | Decimal | float | int) -> Self | Decimal: ...
    def __rtruediv__(self, other: Self | Decimal | float | int) -> Self | Decimal: ...
    def __neg__(self) -> Self: ...
    def __abs__(self) -> Self: ...
    def __bool__(self) -> bool: ...
    def __eq__(self, other: Self) -> bool: ...
    def __lt__(self, other: Self) -> bool: ...
    def __le__(self, other: Self) -> bool: ...
    def __gt__(self, other: Self) -> bool: ...
    def __ge__(self, other: Self) -> bool: ...
    def for_json(self) -> str: ...

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
    def is_equal_up_to_cents(self, other: Self) -> bool: ...
    def is_lower_up_to_cents(self, other: Self) -> bool: ...
    def is_lower_or_equal_up_to_cents(self, other: Self) -> bool: ...
    def rounded_to_cents(self) -> Self: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __hash__(self) -> int: ...
    def __add__(self, other: Self) -> Self: ...
    def __sub__(self, other: Self) -> Self: ...
    def __mul__(self, other: MoneyWithVATRatio | Decimal | float | int) -> Self: ...
    def __rmul__(self, other: MoneyWithVATRatio | Decimal | float | int) -> Self: ...
    def __truediv__(self, other: Decimal | float | int) -> Self: ...
    def __rtruediv__(self, other: Decimal | float | int) -> Self: ...
    def __neg__(self) -> Self: ...
    def __abs__(self) -> Self: ...
    def __bool__(self) -> bool: ...
    def __eq__(self, other: Self) -> bool: ...
    def __lt__(self, other: Self) -> bool: ...
    def __le__(self, other: Self) -> bool: ...
    def __gt__(self, other: Self) -> bool: ...
    def __ge__(self, other: Self) -> bool: ...
    @staticmethod
    def max(*args: object) -> Self | None: ...
    @staticmethod
    def ratio(dividend: Self, divisor: Self) -> MoneyWithVATRatio: ...
    @staticmethod
    def safe_ratio(
        dividend: Self | None, divisor: Self | None
    ) -> MoneyWithVATRatio | None: ...
    @staticmethod
    def safe_ratio_decimal(
        dividend: Self | None,
        divisor: Decimal | None,
    ) -> MoneyWithVATRatio | None: ...
    @staticmethod
    def fast_sum(iterable: Iterable[Self | None]) -> Self: ...
    @staticmethod
    def fast_sum_with_none(iterable: Iterable[Self | None]) -> Self | None: ...
    def for_json(self) -> dict: ...
    @staticmethod
    def from_json(dict: dict) -> Self: ...

class MoneyWithVATRatio:
    def __init__(
        self, net_ratio: Decimal | float | int, gross_ratio: Decimal | float | int
    ) -> None: ...
    @property
    def net_ratio(self) -> Decimal: ...
    @property
    def gross_ratio(self) -> Decimal: ...
    @staticmethod
    def zero() -> Self: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __mul__(self, other: Decimal | float | int) -> Self: ...
    def __eq__(self, other: Self) -> bool: ...
    def for_json(self) -> dict: ...
