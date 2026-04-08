{
  description = "kat preview";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.${system}.default = pkgs.writeShellScriptBin "kat-preview" ''
        echo "Dracula"
      '';
    };
}
