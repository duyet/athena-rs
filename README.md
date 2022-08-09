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
$ cd examples && cargo run apply ./prd
```

# License

MIT.
