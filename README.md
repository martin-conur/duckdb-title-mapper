# DuckDB Title Mapper

`duckdb-title-mapper` is a highly optimized DuckDB extension written in Rust. It standardizes scraped job titles to BLS (Bureau of Labor Statistics) standard titles using a fast TF-IDF implementation.

## Features
- **Rust-based**: Built entirely in Rust for performance and safety.
- **Optimized**: Uses fast TF-IDF for efficient title standardization.
- **DuckDB Integration**: Seamlessly integrates as a DuckDB extension.

## Cloning

Clone the repo with submodules

```shell
git clone --recurse-submodules <repo>
```

## Dependencies
In principle, these extensions can be compiled with the Rust toolchain alone. However, this template relies on some additional
tooling to make life a little easier and to be able to share CI/CD infrastructure with extension templates for other languages:

- Python3
- Python3-venv
- [Make](https://www.gnu.org/software/make)
- Git

Installing these dependencies will vary per platform:
- For Linux, these come generally pre-installed or are available through the distro-specific package manager.
- For MacOS, [homebrew](https://formulae.brew.sh/).
- For Windows, [chocolatey](https://community.chocolatey.org/).

## Building the Extension

To build the extension, ensure you have the following dependencies installed:
- Python3
- Python3-venv
- [Make](https://www.gnu.org/software/make)
- Git

### Steps to Build

1. **Configure the Environment**:
   ```shell
   make configure
   ```
   This sets up a Python virtual environment with DuckDB and its test runner installed. It also determines the correct platform for compilation.

2. **Build the Extension**:
   - For a debug build:
     ```shell
     make debug
     ```
     This produces a shared library in `target/debug/<shared_lib_name>` and transforms it into a loadable DuckDB extension in the `build/debug` directory.

   - For a release build:
     ```shell
     make release
     ```
     This creates an optimized release binary in the `build/release` directory.

## Testing the Extension

This extension uses the DuckDB Python client for testing. The tests are written in the SQLLogicTest format.

- To test the debug build:
  ```shell
  make test_debug
  ```

- To test the release build:
  ```shell
  make test_release
  ```

## Using the Extension in DuckDB

Once the extension is built, you can load it into DuckDB as follows:

1. Start DuckDB:
   ```shell
   duckdb
   ```

2. Load the extension:
   ```sql
   LOAD 'build/release/standarize_title.duckdb_extension';
   ```

3. Use the extension to standardize job titles:
   ```sql
   SELECT standardize_title(scraped_title_column) FROM your_table;
   ```

## Version Switching

To test with a different DuckDB version:

1. Clean the previous configuration:
   ```shell
   make clean_all
   ```

2. Configure with the desired DuckDB version:
   ```shell
   DUCKDB_TEST_VERSION=v1.1.2 make configure
   ```

3. Build and test:
   ```shell
   make debug
   make test_debug
   ```

## Known Issues

On Windows with Python 3.11, the extension may fail to load with the following error:
```shell
IO Error: Extension '<name>.duckdb_extension' could not be loaded: The specified module could not be found
```
This issue is resolved by using Python 3.12.

---
This extension is designed to make job title standardization fast and efficient. Let us know if you encounter any issues or have suggestions for improvement!