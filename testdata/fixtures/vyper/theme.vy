name: public(String[32])

@deploy
def __init__():
    self.name = "Dracula"

@external
@view
def render() -> String[32]:
    return self.name
