{inputs, ...}: {
  perSystem = {
    l,
    pkgs,
    config,
    ...
  }: {
    packages.paper = let
      paper-config = l.fromTOML (l.readFile ./typst.toml);
    in
      pkgs.stdenvNoCC.mkDerivation {
        inherit (paper-config.package) name version;

        src = l.fileset.toSource {
          root = ../.;
          fileset = l.fileset.unions [
            ../Justfile
            ./.
          ];
        };

        buildInputs = l.attrValues {
          inherit (pkgs) typst just;
        };

        HOME = "$(mktemp -d)";

        preBuildPhase =
          if pkgs.stdenv.isDarwin
          then ''
            cp -rv ${inputs.typst-packages} $HOME/Library/Caches/typst
          ''
          else ''
            export XDG_CACHE_HOME = "$HOME/.cache";
            cp -rv ${inputs.typst-packages}/packages $XDG_CACHE_HOME/typst
          '';

        buildPhase = ''
          mkdir out/
          just paper-build
        '';

        installPhase = ''
          cp -r ./out $out
        '';
      };

    devShells.paper = pkgs.mkShell {
      inputsFrom = [config.packages.paper];
      packages = l.attrValues {
        inherit (pkgs) typstyle tinymist;
      };
    };
  };
}
