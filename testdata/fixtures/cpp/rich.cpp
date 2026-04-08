#include <string>

class ThemePreview {
public:
  explicit ThemePreview(std::string name) : name_(std::move(name)) {}

  std::string render() const {
    return R"(theme:dracula)";
  }

private:
  std::string name_;
};

int main() {
  ThemePreview preview("Dracula");
  return static_cast<int>(preview.render().size());
}
