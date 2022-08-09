CREATE EXTERNAL TABLE IF NOT EXISTS `` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_1';

