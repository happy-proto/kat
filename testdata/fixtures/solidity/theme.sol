pragma solidity ^0.8.28;

contract ThemePreview {
    string public constant DEFAULT_THEME = "Dracula";

    function render() external pure returns (string memory) {
        return DEFAULT_THEME;
    }
}
