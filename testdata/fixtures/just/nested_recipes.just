default:
  echo "shell"

python-demo:
  #!/usr/bin/env python3
  class Preview:
      def render(self) -> int:
          return 42
  print(Preview().render())

bash-demo:
  source .env
  echo "$HOME"
