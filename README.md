# athena-rs

[![codecov](https://codecov.io/gh/duyet/athena-rs/branch/master/graph/badge.svg?token=FVVxtMwb4q)](https://codecov.io/gh/duyet/athena-rs)
[![cargo-clippy](https://github.com/duyet/athena-rs/actions/workflows/cargo-clippy.yml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-clippy.yml)
[![cargo-test](https://github.com/duyet/athena-rs/actions/workflows/cargo-test.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-test.yaml)
[![Code coverage](https://github.com/duyet/athena-rs/actions/workflows/cov.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cov.yaml)
[![cargo-doc](https://github.com/duyet/athena-rs/actions/workflows/cargo-doc.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-doc.yaml)
[![cargo-fmt](https://github.com/duyet/athena-rs/actions/workflows/cargo-fmt.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-fmt.yaml)


Athena-rs is a Rust-based tool for managing AWS Athena schemas. This tool provides two commands to build SQL templates and apply them to Athena. 
The build command renders SQL from the specified template path using the [`Tera`](https://tera.netlify.app) template engine, while the apply command builds and executes the SQL in Athena.


# Installation

The following command can be used to install athena-rs:

<!-- BEGIN INSTALLATION -->
```bash
$ cargo install --git https://github.com/duyet/athena-rs
$ athena --help

Managing AWS Athena Schemas

Usage: athena <COMMAND>

Commands:
  build  Build SQL from template path
  apply  Build and execute SQL to Athena
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
<!-- END INSTALLATION -->

# Usages

This is an example of a project structure:

```bash
.
├── base
│   ├── index.sql
│   ├── table_1.sql
│   └── table_2.sql
├── prd
│   └── index.sql
└── stg
    └── index.sql
```

The `prd/index.sql` file is defined as follows:


```sql
{% set s3_bucket = "s3://prd" %}
{% include 'base/index.sql' %}
```

The `base/index.sql` file is defined as follows:

```sql
{% include "base/table_1.sql" %}
{% include "base/table_2.sql" %}
```

The `base/table_1.sql` file is defined as follows:

```sql
CREATE EXTERNAL TABLE IF NOT EXISTS `table_1` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_1';

```

### 1. Build SQL from the specified template

Use the following command to render the template located in [`./prd`](examples/prd):

```bash
$ cd examples && cargo run build ./prd
```

```sql
CREATE EXTERNAL TABLE IF NOT EXISTS `table_1` (
  id string,
  name string
) LOCATION 's3://prd/table_1';


CREATE EXTERNAL TABLE IF NOT EXISTS `table_2` (
  id string,
  name string
) LOCATION 's3://prd/table_2';
```

### 2. Apply SQL to Athena

```bash
$ cd examples && athena apply --output_location=s3://athena-output/ ./prd
```

Compatible with AWS Authentication methods:
<https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-quickstart.html>

Using AWS profile

```bash
$ cat ~/.aws/credentials
# [prd]
# aws_access_key_id=AKIAIOSFODNN7EXAMPLE
# aws_secret_access_key=wJalrXUtnFEMI/K7MDENGb/PxRfiCYEXAMPLEKEY

$ cd examples && athena apply --profile prd --region us-east-1 ./prd
```

Using environment variables

```bash
$ export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
$ export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
$ export AWS_DEFAULT_REGION=us-west-2

$ cd examples && athena apply ./prd
```

# Example templates

- Create Athena View: [./examples/base/view.sql](./examples/base/view.sql)

  ```sql
  -- Database: db1
  CREATE VIEW "view" AS SELECT * FROM table_2;

  /* Database: db2 */
  CREATE VIEW "view" AS SELECT * FROM table_1;
  ```

- Add partitions date range: [./examples/base/table_1_partitions.sql](./examples/base/table_1_partitions.sql)

  ```sql
  ALTER TABLE table_name ADD IF NOT EXISTS
    PARTITION (date_key = "2022-01-01") LOCATION "s3://stg/table_1/year=2022/month=01/day=01",
    PARTITION (date_key = "2022-01-02") LOCATION "s3://stg/table_1/year=2022/month=01/day=02",
    ...
  ```

# Limitations

- The athena apply command currently does not support handling errors that occur when executing the generated SQL statements. You must manually check the Athena console or logs for any errors that may occur during execution.
- This tool has only been tested with basic SQL queries and may not work correctly with more complex queries or with specific versions of AWS Athena.
- Since Athena can run only one query in a session. So `athena apply` break the queries by semicolon `;`.
  Must includes the semicolon `;` at the end of each SQL statement.
- `CREATE VIEW`:
  - As synopsis of Athena do not accept database name. So please add the database name before the query like this example: [view.sql](./examples/view.sql)
  - Backquoted identifiers are not supported, use double quotes to quote identifiers.
- Maybe a bugs of [`walkdir`](https://docs.rs/walkdir), cannot named a file as `macros.sql`

# License

MIT.
