import alasco_money as _money
import pydantic as _pydantic


def test_money_schema():
    class Container(_pydantic.BaseModel):
        value: _money.Money

    assert Container.model_json_schema()["properties"]["value"] == {
        "example": "123.123456789012",
        "title": "Value",
        "type": "string",
    }


def test_money_vat_schema():
    class Container(_pydantic.BaseModel):
        value: _money.MoneyWithVAT

    assert Container.model_json_schema()["properties"]["value"] == {
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
        "title": "Value",
        "type": "object",
    }


def test_money_vat_ratio_schema():
    class Container(_pydantic.BaseModel):
        value: _money.MoneyWithVATRatio

    assert Container.model_json_schema()["properties"]["value"] == {
        "properties": {
            "net_ratio": {"title": "Net ratio", "type": "string", "example": "0.23"},
            "gross_ratio": {
                "title": "Gross ratio",
                "type": "string",
                "example": "0.23",
            },
        },
        "title": "Value",
        "type": "object",
    }
