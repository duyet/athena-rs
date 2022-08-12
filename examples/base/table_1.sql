/* Database: db */
CREATE EXTERNAL TABLE IF NOT EXISTS `table1` (
  id string,
  name string
) LOCATION '{{ s3_bucket }}/table_1';

