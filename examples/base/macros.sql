{% macro create_db(name) %}
CREATE DATABASE IF NOT EXISTS {{ name }};
{% endmacro create_db %}
