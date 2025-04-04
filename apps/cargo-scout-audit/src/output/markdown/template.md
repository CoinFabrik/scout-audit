{% if render_styles == true %}

<style>
.markdown-body table {min-width: 100%;width: 100%;display: table;}
thead {min-width: 100%;width: 100%;}
th {min-width: 60%;width: 60%;}
th:last-child {min-width: 20%;width: 20%;}
th:first-child {min-width: 20%;width: 20%;}
</style>

{% endif %}

# Scout Report - {{ report.name }} - {{ report.date }}

## Summary

| {% for col in summary_table.header_order %}{{ filter_cell(cell=summary_table.header[col]) }} | {% endfor %}
| {% for col in summary_table.header_order %}- | {% endfor %}
{% for row in summary_table.rows -%}
| {% for col in summary_table.header_order %}{{ filter_cell(cell=row[col]) }} | {% endfor %}
{% endfor %}

Issues found:

{% for category in summary.categories %}

- [{{ category.name }}](#{{ category.link }}) ({{ category.results_count }} results) ({{ category.severity }}){% endfor %}

{% for category in report.categories %}

## {{ category.id }}

{% for vulnerability in category.vulnerabilities %}

### {{ vulnerability.name }}

**Impact:** {{ vulnerability.severity | capitalize }}

**Issue:** {{ vulnerability.short_message }}

**Description:** {{ vulnerability.long_message }}

[**Learn More**]({{ vulnerability.help }})

#### Findings

| ID  | Package | File Location |
| --- | ------- | ------------- |
{% for finding in report.findings -%}
{% if finding.category_id == category.id and finding.vulnerability_id == vulnerability.id -%}
| {{ finding.id }} | {{ finding.package }} | {% if render_styles %}[{{ finding.span }}]({{ finding.file_path }}){% else %}{{ finding.span }}{% endif %} |
{% endif -%}
{% endfor -%}

{% endfor %}
{% endfor %}
