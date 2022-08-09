CREATE EXTERNAL TABLE IF NOT EXISTS `table_1` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_1';

