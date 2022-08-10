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

# License

MIT.
