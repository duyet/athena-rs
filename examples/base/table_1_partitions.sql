/* Database: db */
-- Add partitions
ALTER TABLE table_1 ADD IF NOT EXISTS
{% for date_str in date_range(start = "2022-01-01", end = "2022-01-10") %}
{%- set year = date_str | date(format = "%Y") -%}
{%- set month = date_str | date(format = "%m") -%}
{%- set day = date_str | date(format = "%d") -%}
PARTITION (date_key = "{{ date_str }}") LOCATION "{{ s3_bucket }}/table_1/year={{ year }}/month={{ month }}/day={{ day }}" 
{%- if loop.last %};{% endif %}
{% endfor %}
