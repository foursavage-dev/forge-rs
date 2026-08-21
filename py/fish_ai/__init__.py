# Lazy re-export keeps `from fish_ai import FishAIServer` working without
# eagerly importing `fish_ai.server`. That matters because running
# `python -m fish_ai.server` first imports this package: an eager `from .server
# import ...` here would leave `fish_ai.server` pre-loaded in `sys.modules` and
# make runpy emit a RuntimeWarning.
def __getattr__(name):
    if name == "FishAIServer":
        from .server import FishAIServer

        return FishAIServer
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
