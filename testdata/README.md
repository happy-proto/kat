# Test Data

This directory keeps repository-local sample files for both automated tests and
manual rendering checks.

## Layout

- `fixtures/`: small and stable inputs for automated tests.
- `perf/`: repository-local performance baseline manifests.
- `showcase/`: richer examples for visually checking syntax highlighting output.

## Conventions

- Keep each fixture focused on one behavior or detection rule.
- Add a dedicated fixture for every supported nested-highlighting scenario instead of
  relying only on broad showcase samples.
- When a nested scenario becomes stable, also add at least one showcase file so it can be
  visually checked during theme/query work.
- Prefer realistic source text over synthetic token soup.
- Add new showcase files when a language, theme rule, or injection scenario is
  hard to evaluate from tiny fixtures.
- Showcase files may include currently unsupported languages if they help define
  the future rendering quality bar.
