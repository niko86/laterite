"""Coverage campaign P5: `laterite.dynamic` — the on-demand typed-graph class
factory `read_typed` uses for custom AGS groups not in the compiled registry.

Behavioural: a registered class must construct/validate/repr/walk like the
compiled `#[pyclass]` types, cache by shape, disambiguate a same-code/different-
shape conflict, and stay importable via the module's PEP 562 `__getattr__`.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from laterite import dynamic

if TYPE_CHECKING:
    from collections.abc import Iterator

_HEADINGS = [{"name": "MYC_ID", "type": "ID"}, {"name": "MYC_VAL", "type": "2DP"}]
_HEADINGS_ALT = [{"name": "MYC_ID", "type": "ID"}, {"name": "MYC_NOTE", "type": "X"}]


@pytest.fixture(autouse=True)
def _isolated_registry() -> Iterator[None]:
    dynamic.clear_cache()
    yield
    dynamic.clear_cache()


def test_register_instantiate_repr_walk() -> None:
    cls = dynamic.get_or_register("MYC", _HEADINGS)
    assert cls.__name__ == "MYC"
    assert cls.__module__ == "laterite.dynamic"

    inst = cls(myc_id="X1", myc_val=1.5)
    assert inst.myc_id == "X1"
    assert inst.myc_val == 1.5
    assert repr(inst) == "MYC(...)"
    assert inst.walk("MYC") == []  # dynamic classes are passthrough leaves

    # a missing field defaults to None (kwargs contract)
    assert cls().myc_id is None


def test_unknown_kwarg_raises_typeerror() -> None:
    cls = dynamic.get_or_register("MYC", _HEADINGS)
    with pytest.raises(TypeError, match="unexpected keyword"):
        cls(not_a_field="x")


def test_same_shape_returns_cached_class() -> None:
    a = dynamic.get_or_register("MYC", _HEADINGS)
    b = dynamic.get_or_register("myc", _HEADINGS)  # code is upper-cased
    assert a is b  # the (code, shape) cache hit


def test_conflicting_shape_is_disambiguated() -> None:
    a = dynamic.get_or_register("MYC", _HEADINGS)
    b = dynamic.get_or_register("MYC", _HEADINGS_ALT)
    assert a is not b
    assert a.__name__ == "MYC"
    # the second shape gets a stable 8-char hash suffix, both coexist
    assert b.__name__.startswith("MYC__")
    assert len(b.__name__) == len("MYC__") + 8

    reg = dynamic.registered_classes()
    assert set(reg) == {"MYC"}
    assert len(reg["MYC"]) == 2


def test_module_getattr_and_dir() -> None:
    cls = dynamic.get_or_register("MYC", _HEADINGS)
    # PEP 562 module __getattr__ — the `from laterite.dynamic import MYC` door.
    # A variable (not a constant) so this genuinely goes through __getattr__.
    name = "MYC"
    assert getattr(dynamic, name) is cls
    assert "MYC" in dir(dynamic)
    assert "get_or_register" in dir(dynamic)  # __all__ still present
    with pytest.raises(AttributeError, match="no class with that name"):
        _ = dynamic.THIS_IS_NOT_REGISTERED
