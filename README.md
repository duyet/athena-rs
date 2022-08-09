# athena-rs
Managing AWS Athena Schemas

# Cli Installation

```bash
cargo install https://github.com/duyet/athena-rs
```

# Usages

Example project structure

```
.
├── base
│   ├── index.sql
│   └── table_1.sql
├── prd
│   └── index.sql
└── stg
    └── index.sql
```

File `base/index.sql`:

```sql
{% include './table_1.sql' %}
```

File `prd/index.sql`:

```sql
{% include '../base.sql' %}
```
