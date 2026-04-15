pragma solidity ^0.8.28;

error ThemeNotFound(bytes32 slug);

contract ThemeRegistry {
    struct Theme {
        string name;
        bool active;
    }

    mapping(bytes32 => Theme) private themes;

    constructor() {
        themes[keccak256("dracula")] = Theme("Dracula", true);
    }

    function themeBySlug(bytes32 slug) external view returns (Theme memory) {
        Theme memory theme = themes[slug];
        if (bytes(theme.name).length == 0) {
            revert ThemeNotFound(slug);
        }
        return theme;
    }
}
