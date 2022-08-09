CREATE EXTERNAL TABLE IF NOT EXISTS `table_2` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_2';

