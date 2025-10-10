# Scripts

Utility scripts for managing the DuckDB extension.

## set_duckdb_version.sh

Updates the DuckDB version across all project files in a single command.

### Usage

```bash
# Direct usage
./scripts/set_duckdb_version.sh v1.4.0

# Via Makefile (recommended)
make set-version VERSION=v1.4.0
```

### What it updates

- **Makefile**
  - `TARGET_DUCKDB_VERSION=v1.4.0`
  - `DUCKDB_TEST_VERSION=1.4.0`

- **Cargo.toml**
  - `duckdb = { version = "1.4.0", ... }`
  - `libduckdb-sys = { version = "1.4.0", ... }`

- **Cargo.lock**
  - Runs `cargo update -p duckdb -p libduckdb-sys`

- **GitHub Actions** (.github/workflows/MainDistributionPipeline.yml)
  - `uses: duckdb/extension-ci-tools/.github/workflows/_extension_distribution.yml@v1.4.0`
  - `duckdb_version: v1.4.0`
  - `DUCKDB_VERSION="v1.4.0"`

### Examples

```bash
# Upgrade to v1.5.0
make set-version VERSION=v1.5.0

# Downgrade to v1.3.0 for compatibility testing
make set-version VERSION=v1.3.0

# Return to current stable
make set-version VERSION=v1.4.0
```

### After running

1. Review changes: `git diff`
2. Clean build: `make clean_all`
3. Reconfigure: `make configure`
4. Test locally: `make debug && make test_debug`
5. Commit: `git add -A && git commit -m "Bump DuckDB to v1.5.0"`
6. Push to trigger CI: `git push origin master`

The CI pipeline will automatically build binaries for the new version and store them in `builds/v1.5.0/`.
