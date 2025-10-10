.PHONY: clean clean_all set-version

PROJ_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

EXTENSION_NAME=title_mapper

# Set to 1 to enable Unstable API (binaries will only work on TARGET_DUCKDB_VERSION, forwards compatibility will be broken)
# Note: currently extension-template-rs requires this, as duckdb-rs relies on unstable C API functionality
USE_UNSTABLE_C_API=1

# Target DuckDB version
TARGET_DUCKDB_VERSION=v1.4.1
DUCKDB_TEST_VERSION=1.4.1

all: configure debug

# Include makefiles from DuckDB
include extension-ci-tools/makefiles/c_api_extensions/base.Makefile
include extension-ci-tools/makefiles/c_api_extensions/rust.Makefile

configure: venv platform extension_version

debug: build_extension_library_debug build_extension_with_metadata_debug
release: build_extension_library_release build_extension_with_metadata_release

test: test_debug
test_debug: test_extension_debug
test_release: test_extension_release

clean: clean_build clean_rust
clean_all: clean_configure clean

# Version management helper
set-version:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make set-version VERSION=v1.4.0"; \
		echo ""; \
		echo "This will update DuckDB version in:"; \
		echo "  - Makefile"; \
		echo "  - Cargo.toml"; \
		echo "  - Cargo.lock"; \
		echo "  - GitHub Actions workflow"; \
		exit 1; \
	fi
	@bash scripts/set_duckdb_version.sh $(VERSION)
