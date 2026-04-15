(comment) @comment

(at_rule
  keyword: (at_keyword) @keyword)

(at_rule
  value: (line_value) @string)

(variable_assignment
  name: (css_variable) @variable)

(variable_assignment
  name: (identifier) @variable)

(variable_assignment
  value: (line_value) @string)

(property_line
  name: (property_name) @property)

(property_line
  value: (line_value) @string)

(selector_line
  selector: (selector) @tag)

"=" @operator
