{
  description = "VtuberVRMTool 개발 환경";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "vtuber-vrm-tool-dev-shell";

          packages = with pkgs; [
            dotnet-sdk_8
            mono
            msbuild
            git
            ripgrep
            tree
          ];

          shellHook = ''
            echo "[VtuberVRMTool] Nix 개발 환경에 진입했습니다."
            echo "Unity 프로젝트에서 VRM 에디터 스크립트를 편집/검토할 수 있습니다."
          '';
        };
      });
}
