def preview($name):
  {
    theme: ($name // "Dracula"),
    enabled: true,
    count: 3
  };

.themes[]
| select(.enabled == true)
| preview(.name)
