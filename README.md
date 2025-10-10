# DuckDB Title Mapper

`duckdb-title-mapper` is a highly optimized DuckDB extension written in Rust. It standardizes scraped job titles to BLS (Bureau of Labor Statistics) standard titles using a fast TF-IDF implementation.

## What It Does

This extension transforms messy, inconsistent job titles from various sources into standardized BLS titles:

| Scraped Title (Input) | Standardized Title (Output) |
|------------------------|------------------------------|
| Sr. Software Eng | Software Engineer |
| Registered Nurse - ICU | Registered Nurse |
| Accountant III | Accountant |
| Sales Rep (B2B) | Sales Representative |
| Elementary School Teacher - 3rd Grade | Elementary School Teacher |
| Exec. Chef | Executive Chef |
| Marketing Coordinator/Specialist | Marketing Specialist |
| Licensed Practical Nurse (LPN) | Licensed Practical Nurse |

**Output Format:** The extension returns results in the format: `Specific Title - BLS Standard Category`

This makes it easy to:
- **Group** similar job titles together
- **Analyze** job market trends consistently
- **Filter** job postings by standardized categories
- **Compare** titles across different companies and sources
- **Extract** either the specific title or BLS category using SQL string functions

## Features
- **Rust-based**: Built entirely in Rust for performance and safety.
- **Optimized**: Uses fast TF-IDF for efficient title standardization.
- **DuckDB Integration**: Seamlessly integrates as a DuckDB extension.

## Table of Contents
- [Quick Start](#quick-start)
- [Cloning](#cloning)
- [Dependencies](#dependencies)
- [Building the Extension](#building-the-extension)
- [Testing the Extension](#testing-the-extension)
- [Using the Extension in DuckDB](#using-the-extension-in-duckdb)
  - [Usage Examples](#usage-examples)
- [Version Management](#version-management)
  - [Automated Version Updates](#automated-version-updates-recommended)
  - [Manual Version Switching](#manual-version-switching-alternative)
- [Common Workflows](#common-workflows)
  - [Development Workflow](#development-workflow)
  - [Release Build Workflow](#release-build-workflow)
  - [Clean Build Workflow](#clean-build-workflow)
- [Troubleshooting](#troubleshooting)
- [Known Issues](#known-issues)

## Quick Start

If you just want to get up and running quickly:

```shell
# 1. Clone with submodules
git clone --recurse-submodules https://github.com/martin-conur/duckdb-title-mapper
cd duckdb-title-mapper

# 2. Configure and build
make configure
make debug

# 3. Test it
make test_debug

# 4. Use it in DuckDB
duckdb
```

```sql
-- Load the extension
LOAD 'build/debug/standardize_title.duckdb_extension';

-- Try it out with different job titles
SELECT standardize_title('Senior Software Engineer');
SELECT standardize_title('Registered Nurse - ICU');
SELECT standardize_title('Sales Manager');
```

## Cloning

Clone the repo with submodules

```shell
git clone --recurse-submodules https://github.com/martin-conur/duckdb-title-mapper
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
   LOAD 'build/release/standardize_title.duckdb_extension';
   ```

3. Use the extension to standardize job titles:
   ```sql
   SELECT standardize_title(scraped_title_column) FROM your_table;
   ```

### Usage Examples

Here are some practical examples of using the extension across different industries:

#### Example 1: Basic Title Standardization
```sql
-- Standardize tech job titles
SELECT standardize_title('Sr. Software Eng') AS standardized_title;
-- Result: 'Software Engineer - Software Developers'

-- Standardize healthcare titles
SELECT standardize_title('RN - Emergency Room') AS standardized_title;
-- Result: 'Registered Nurse - Registered Nurses'

-- Standardize education titles
SELECT standardize_title('Teacher - High School Math') AS standardized_title;
-- Result: 'High School Teacher - Secondary School Teachers'
```

#### Example 2: Bulk Processing from a Table
```sql
-- Create a sample table with scraped job titles from various industries
CREATE TABLE job_postings (
    id INTEGER,
    original_title VARCHAR,
    company VARCHAR,
    industry VARCHAR
);

INSERT INTO job_postings VALUES
    (1, 'Sr Software Engineer - Backend', 'TechCorp', 'Technology'),
    (2, 'Registered Nurse (ICU)', 'City Hospital', 'Healthcare'),
    (3, 'Sales Associate - Retail', 'ShopMart', 'Retail'),
    (4, 'Accountant II', 'Finance Plus', 'Finance'),
    (5, 'Executive Chef', 'Fine Dining Inc', 'Hospitality'),
    (6, 'Marketing Coordinator/Manager', 'AdAgency', 'Marketing');

-- Standardize all titles
SELECT
    id,
    original_title,
    standardize_title(original_title) AS standardized_title,
    industry,
    company
FROM job_postings;
```

#### Example 3: Grouping and Aggregation
```sql
-- Count jobs by standardized title across industries
SELECT
    standardize_title(original_title) AS standard_title,
    COUNT(*) AS job_count,
    COUNT(DISTINCT industry) AS industries_count
FROM job_postings
GROUP BY standardize_title(original_title)
ORDER BY job_count DESC;
```

#### Example 4: Filtering with Standardized Titles
```sql
-- Find all nursing position variations
SELECT
    original_title,
    standardize_title(original_title) AS standardized_title,
    company
FROM job_postings
WHERE standardize_title(original_title) IN ('Registered Nurse', 'Licensed Practical Nurse', 'Nurse Practitioner');

-- Find all engineer variations
SELECT
    original_title,
    standardize_title(original_title) AS standardized_title
FROM job_postings
WHERE standardize_title(original_title) LIKE '%Engineer%';
```

#### Example 5: Extracting Specific Title and BLS Category
```sql
-- Extract both the specific title and BLS category separately
SELECT
    original_title,
    standardize_title(original_title) AS full_standardized,
    split_part(standardize_title(original_title), ' - ', 1) AS specific_title,
    split_part(standardize_title(original_title), ' - ', 2) AS bls_category
FROM job_postings;

-- Example output:
-- original_title              | full_standardized                                    | specific_title      | bls_category
-- Sr Software Engineer        | Software Engineer - Software Developers              | Software Engineer   | Software Developers
-- Registered Nurse (ICU)      | Registered Nurse - Registered Nurses                 | Registered Nurse    | Registered Nurses

-- Group by BLS category
SELECT
    split_part(standardize_title(original_title), ' - ', 2) AS bls_category,
    COUNT(*) AS job_count
FROM job_postings
GROUP BY bls_category
ORDER BY job_count DESC;
```

#### Example 6: Creating Views with Standardized Data
```sql
-- Create a view with standardized titles for easier querying
CREATE VIEW standardized_jobs AS
SELECT
    id,
    original_title,
    standardize_title(original_title) AS standardized_title,
    split_part(standardize_title(original_title), ' - ', 1) AS specific_title,
    split_part(standardize_title(original_title), ' - ', 2) AS bls_category,
    industry,
    company
FROM job_postings;

-- Query healthcare positions
SELECT * FROM standardized_jobs
WHERE industry = 'Healthcare';

-- Query by BLS category across all industries
SELECT * FROM standardized_jobs
WHERE bls_category = 'Software Developers';
```

## Version Management

### Automated Version Updates (Recommended)

The project includes a version management script that updates DuckDB versions across all project files automatically:

```shell
# Update to a specific DuckDB version
make set-version VERSION=v1.4.0
```

This command will update:
- **Makefile** (`TARGET_DUCKDB_VERSION` and `DUCKDB_TEST_VERSION`)
- **Cargo.toml** (duckdb and libduckdb-sys dependencies)
- **Cargo.lock** (via `cargo update`)
- **GitHub Actions workflow** (.github/workflows/MainDistributionPipeline.yml)

#### Version Update Workflow Example

```shell
# 1. Update to a new DuckDB version (use an actual released version)
make set-version VERSION=v1.4.0

# 2. Review the changes
git diff

# 3. Clean and rebuild with new version
make clean_all
make configure
make debug

# 4. Test the new version
make test_debug

# 5. Commit if everything works
git add -A
git commit -m "Bump DuckDB to v1.4.0"
```

> **Note:** Replace `v1.4.0` with any [officially released DuckDB version](https://github.com/duckdb/duckdb/releases). Future versions like v1.5.0 can be used once they are released.

See [scripts/README.md](scripts/README.md) for more details on version management.

### Manual Version Switching (Alternative)

To manually test with a different DuckDB version:

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

> **Note:** Using the automated `make set-version` command is recommended as it ensures all files stay in sync.

## Common Workflows

### Development Workflow

```shell
# Make code changes to src/
# ... edit files ...

# Rebuild and test
make debug
make test_debug

# If tests pass, try it in DuckDB
duckdb
```

### Release Build Workflow

```shell
# Build optimized version
make release

# Test the release build
make test_release

# Use the release build
duckdb
```

```sql
LOAD 'build/release/standardize_title.duckdb_extension';

-- Test with various job titles
SELECT standardize_title('Software Developer III');
SELECT standardize_title('Nursing Supervisor');
SELECT standardize_title('Senior Accountant CPA');
```

### Clean Build Workflow

If you encounter build issues or want to start fresh:

```shell
# Clean build artifacts only
make clean

# Clean everything (including configuration)
make clean_all

# Reconfigure and rebuild
make configure
make debug
```

## Troubleshooting

### Extension Won't Load

**Problem:** Extension fails to load in DuckDB.

**Solution:**
```shell
# Ensure you've built the extension first
make debug  # or make release

# Check the file exists
ls -la build/debug/standardize_title.duckdb_extension
```

### Build Failures

**Problem:** Build fails with compilation errors.

**Solution:**
```shell
# Clean and rebuild
make clean_all
make configure
make debug

# Check Rust toolchain is installed
cargo --version
```

### Version Mismatch Issues

**Problem:** Extension built for wrong DuckDB version.

**Solution:**
```shell
# Update to correct version
make set-version VERSION=v1.4.1

# Clean and rebuild
make clean_all
make configure
make debug
```

### Python Virtual Environment Issues

**Problem:** `make configure` fails.

**Solution:**
```shell
# Ensure Python3 and venv are installed
python3 --version
python3 -m venv --help

# Try reconfiguring
make clean_configure
make configure
```

## Known Issues

On Windows with Python 3.11, the extension may fail to load with the following error:
```shell
IO Error: Extension '<name>.duckdb_extension' could not be loaded: The specified module could not be found
```
This issue is resolved by using Python 3.12.

---
This extension is designed to make job title standardization fast and efficient. Let us know if you encounter any issues or have suggestions for improvement!