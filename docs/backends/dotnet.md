# .NET Backend Guide

> 🌐 **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

Forge integrates with .NET solutions (`*.sln`) and C# projects (`*.csproj`).

---

## Detection & Tasks

Forge identifies .NET projects by scanning for `*.csproj`, `*.fsproj`, or `*.sln` files.

### Mapped Tasks:
- `build`: `dotnet build --no-restore`
- `check`: `dotnet format --verify-no-changes`
- `test`: `dotnet test --no-build`

Artifacts in `bin/` and `obj/` are tracked and managed within the unified cache.
