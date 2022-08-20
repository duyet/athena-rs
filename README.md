# athena-rs

[![codecov](https://codecov.io/gh/duyet/athena-rs/branch/master/graph/badge.svg?token=FVVxtMwb4q)](https://codecov.io/gh/duyet/athena-rs)
[![cargo-clippy](https://github.com/duyet/athena-rs/actions/workflows/cargo-clippy.yml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-clippy.yml)
[![cargo-test](https://github.com/duyet/athena-rs/actions/workflows/cargo-test.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-test.yaml)
[![Code coverage](https://github.com/duyet/athena-rs/actions/workflows/cov.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cov.yaml)
[![cargo-doc](https://github.com/duyet/athena-rs/actions/workflows/cargo-doc.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-doc.yaml)
[![cargo-fmt](https://github.com/duyet/athena-rs/actions/workflows/cargo-fmt.yaml/badge.svg)](https://github.com/duyet/athena-rs/actions/workflows/cargo-fmt.yaml)


Managing AWS Athena Schemas

# Installation

<!-- BEGIN INSTALLATION -->
```bash
$ cargo install --git https://github.com/duyet/athena-rs
$ athena --help

athena 0.1.0
Duyet <me@duyet.net>
Managing AWS Athena Schemas

USAGE:
    athena <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    apply    Build and execute SQL to Athena
    build    Build SQL from template path
    help     Print this message or the help of the given subcommand(s)
```
<!-- END INSTALLATION -->

# Usages

Example project structure

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

File `prd/index.sql`:

```sql
{% set s3_bucket = "s3://prd" %}
{% include 'base/index.sql' %}
```

File `base/index.sql`:

```sql
{% include "base/table_1.sql" %}
{% include "base/table_2.sql" %}
```

File `base/table_1.sql`:

```sql
CREATE EXTERNAL TABLE IF NOT EXISTS `table_1` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_1';

```

### 1. Build SQL from template

Render for `./prd`. This is using [`Tera`](https://tera.netlify.app) template engine
which is inspired by Jinja2 and Django templates.

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

- Since Athena can run only one query in a session. So `athena apply` break the queries by semicolon `;`.
  Must includes the semicolon `;` at the end of each SQL statement.
- `CREATE VIEW`:
  - As synopsis of Athena do not accept database name. So please add the database name before the query like this example: [view.sql](./examples/view.sql)
  - Backquoted identifiers are not supported, use double quotes to quote identifiers.
- Maybe a bugs of [`walkdir`](https://docs.rs/walkdir), cannot named a file as `macros.sql`

# License

MIT.
