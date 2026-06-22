# Contribuir a Dueo

Gracias por tu interés. Este documento explica **cómo se trabaja en el repo**: el
modelo de ramas, cómo se abren los Pull Requests, el formato de los commits y cómo
se publican las versiones (automático). Está pensado tanto para el mantenedor como
para colaboradores externos.

---

## 1. Modelo de ramas (trunk-based)

Dueo usa un modelo **trunk-based**, no el "Gitflow clásico":

- **`master`** es la **única rama de larga vida**. Siempre debe estar en verde
  (compila, pasa lint/tests) y siempre es "releasable".
- El trabajo se hace en **ramas cortas** que salen de `master`, se integran por
  **Pull Request** y se borran al mergear:
  - `feat/...` — una funcionalidad nueva.
  - `fix/...` — un arreglo de bug.
  - `refactor/...`, `docs/...`, `chore/...`, `ci/...` — limpieza, docs, tooling.
- **No hay rama `develop` ni ramas `release/*` ni `hotfix/*`.** El versionado y las
  releases los gestiona [release-please](#5-releases-automáticas) directamente sobre
  `master`; una rama `develop` pelearía con ese flujo. Un "hotfix" es simplemente una
  rama `fix/...` que entra por PR y genera un release de patch.

> **¿Por qué no Gitflow clásico?** Gitflow (con `develop`, `release/*`, `hotfix/*`)
> se diseñó para releases manuales y con calendario. Aquí la release es automática y
> continua (cada `feat`/`fix` que entra a `master` puede convertirse en versión), así
> que trunk-based + PRs cortos es más simple y encaja con la automatización.

---

## 2. Cómo funciona un repo público (fork & PR)

- **Colaboradores externos** (sin permiso de escritura): **no pueden** hacer push al
  repo. El flujo es **fork & pull request**:
  1. Haces **fork** del repo a tu cuenta.
  2. Clonas tu fork, creas una rama, commiteas y haces push **a tu fork**.
  3. Abres un **Pull Request** desde tu fork hacia `master` de este repo.
  4. El mantenedor revisa, la CI corre, y si todo va bien se mergea.
- **Colaboradores con escritura** (añadidos en Settings → Collaborators): pueden crear
  ramas en el propio repo, pero `master` está **protegido** → también entran por PR.
- **El mantenedor** trabaja igual: rama → PR. La protección aplica a todos (incluido
  admin) para mantener la disciplina y que la CI valide siempre.

---

## 3. Flujo paso a paso

```bash
# 1. Parte de master actualizado
git checkout master
git pull

# 2. Crea una rama corta y descriptiva
git checkout -b feat/exportar-csv

# 3. Trabaja y commitea con Conventional Commits (ver §4)
git commit -m "feat(data): export subscriptions as CSV"

# 4. Sube la rama
git push -u origin feat/exportar-csv   # (o a tu fork si eres externo)

# 5. Abre el Pull Request hacia master en GitHub
#    - La CI (ci.yml) corre sola: front (check/lint/build) + server (fmt/clippy/build).
#    - Resuelve los comentarios de review.

# 6. Merge (squash). Borra la rama. release-please se encarga del resto.
```

**Estrategia de merge: _squash_.** Cada PR se integra como **un solo commit** en
`master`. Esto mantiene el historial limpio y un changelog legible. Como el commit del
squash toma el **título del PR**, **el título del PR debe seguir Conventional Commits**
(ver §4). Mantén los PRs **pequeños y centrados** en un solo cambio.

---

## 4. Conventional Commits

El mensaje (o el **título del PR** si se hace squash) sigue el formato:

```
<tipo>(<scope opcional>): <descripción en imperativo>
```

Tipos y su efecto en la versión (SemVer, pre-1.0):

| Tipo | Para qué | ¿Genera release? |
| --- | --- | --- |
| `feat` | funcionalidad nueva | **sí** → sube *minor* (0.x.0) |
| `fix` | arreglo de bug | **sí** → sube *patch* (0.0.x) |
| `refactor` | cambio interno sin alterar comportamiento | no |
| `docs` | solo documentación | no |
| `chore` | mantenimiento/tooling | no |
| `ci` | workflows/CI | no |
| `test` | tests | no |
| `perf` | mejora de rendimiento | sí (patch) |

**Cambios incompatibles (breaking):** añade `!` tras el tipo **o** un footer
`BREAKING CHANGE:`. Pre-1.0 esto sube *minor*; post-1.0 subiría *major*.

```
feat!: rename the import format

BREAKING CHANGE: backups from <0.4 must be re-exported.
```

Scopes habituales: `web`, `server`, `data`, `auth`, `scheduler`, `i18n`, `ui`…
(opcionales, pero ayudan al changelog).

---

## 5. Releases automáticas (release-please)

No se taggea ni se publica a mano. El flujo es:

1. Se mergean a `master` PRs con `feat:`/`fix:` (entre otros).
2. **release-please** (workflow `release.yml`) mantiene un **"release PR"** abierto que
   acumula los cambios: sube la versión en `version.txt`, `dueo-server/Cargo.toml` y
   `dueo-web/package.json`, y actualiza el `CHANGELOG.md`.
3. Cuando el mantenedor **mergea ese release PR**, en el mismo run se crea el **tag
   `vX.Y.Z`** + la **GitHub Release**, y se construye y publica la **imagen Docker**
   multi-arch en `ghcr.io/yelt-dev/dueo` (`:vX.Y.Z`, `:X.Y`, `:latest`).

Es decir: tú decides *cuándo* publicar (mergeando el release PR); el *qué* y el
*número* los calcula release-please desde los commits.

**Hotfix:** rama `fix/...` → PR → merge → release-please propone un patch → mergeas su
PR → release. Sin ramas especiales.

Actualizar una instancia ya desplegada (no pierde datos, la base vive en el volumen):

```bash
docker compose pull && docker compose up -d
```

---

## 6. Proteger `master` (configuración en GitHub)

Esto se configura **en GitHub** (no por código): Settings → **Rules → Rulesets** →
*New branch ruleset* (o Settings → Branches → branch protection clásico).

- **Target:** `master`.
- **Require a pull request before merging** → activado.
  - **Required approvals:** `1` si hay más de un colaborador. **Si eres el único
    mantenedor, déjalo en `0`** (no puedes aprobar tu propio PR; con 1 te bloquearías),
    pero mantén el requisito de PR + checks.
- **Require status checks to pass** → activado, y selecciona los checks **`web`** y
  **`server`** (del workflow `ci.yml`). *Ojo:* solo aparecen en la lista después de que
  hayan corrido al menos una vez (abre un PR primero).
- **Require branches to be up to date before merging** → recomendado.
- **Require linear history** → recomendado (encaja con squash).
- **Block force pushes** y **restrict deletions** → activado.
- **Bypass list / "include administrators":** decídelo. Para máxima disciplina, sin
  bypass (aplica también al admin).

> Ya activado aparte (necesario para que release-please abra su PR):
> Settings → Actions → General → **Workflow permissions = Read and write** +
> **Allow GitHub Actions to create and approve pull requests**.

Recomendado también en Settings → General → Pull Requests: **Allow squash merging** y
*"Default to pull request title for squash merge commits"*; y **Automatically delete
head branches** tras mergear.

---

## 7. Antes de abrir el PR (checklist local)

```bash
# Front
cd dueo-web && pnpm install && pnpm check && pnpm lint && pnpm build

# Server (el front debe estar construido antes: rust-embed lo necesita)
cd ../dueo-server && cargo fmt --check && cargo clippy -- -D warnings && cargo build
```

Convenciones del código: **comentarios y mensajes de log/errores del backend en
inglés**; el texto **visible para el usuario** va por i18n (no hardcodeado). No edites
las migraciones `.sql` ya aplicadas (sqlx valida un checksum por archivo).
