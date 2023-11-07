{% for var in messages -%}
{% if var.t == "Int" -%}{{ var.name }} = gets.to_i
{% elif var.t == "Float" -%}{{ var.name }} = gets.to_f
{% elif var.t == "Long" %}{{ var.name }} = gets.to_i
{% elif var.t == "Bool" %}{{ var.name }} = gets
{% elif var.t == "Word" %}{{ var.name }} = gets
{% elif var.t == "String" %}{{ var.name }} = gets
{% else %}Unknown type {{ var.t }} for {{ var.name }}
{% endif -%}
{% endfor -%}