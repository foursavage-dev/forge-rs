# Java Backend Guide

> Ã°Å¸Å’Â **Translations & Contributions:** Want to translate or improve this document in your language? See our [Translation Guidelines](../../TRANSLATION.md).

Fish supports Java workspaces configured with Apache Maven (`pom.xml`) or Gradle (`build.gradle` / `build.gradle.kts`).

---

## Detection & Tasks

Fish identifies Java packages by detecting `pom.xml` or `build.gradle` manifests.

### Mapped Tasks:
- `build`: `mvn compile` or `gradle assemble`
- `check`: `mvn checkstyle:check` or `gradle check`
- `test`: `mvn test` or `gradle test`

Outputs (such as `.jar` and `.class` trees) are fingerprinted and cached in the local CAS.
