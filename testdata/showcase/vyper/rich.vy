owner: public(address)
theme_count: public(uint256)

@deploy
def __init__():
    self.owner = msg.sender
    self.theme_count = 1

@external
def set_owner(next_owner: address):
    assert msg.sender == self.owner
    self.owner = next_owner

@external
@view
def render(name: String[32]) -> String[64]:
    return concat(name, " preview")
