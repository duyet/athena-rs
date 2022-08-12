# athena-rs

Managing AWS Athena Schemas

# Cli Installation

```bash
$ cargo install https://github.com/duyet/athena-rs
$ athena --help
```

# Usages

Example project structure

```
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

- Since Athena can run only one query in a session. `athena apply` break the queries by comma `;`.
  Must includes the comma (`,`) at the end of each query.
- `CREATE VIEW`:
  - As synopsis of Athena do not accept database name. So please add the database name before the query like this example: [view.sql](./examples/view.sql)
  - Backquoted identifiers are not supported, use double quotes to quote identifiers.

# License

MIT.
