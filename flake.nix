# https://fasterthanli.me/series/building-a-rust-service-with-nix/part-10
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # https://github.com/NixOS/nixpkgs/blob/c9e5b2da15d68680c7b697bb6ecb1d2f86dfc6d3/lib/systems/examples.nix#L248
        pkgs = (import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        });
        pkgsCross = pkgs.pkgsCross.aarch64-embedded;
      in {
        # https://nix.dev/tutorials/cross-compilation.html#developer-environment-with-a-cross-compiler
        # https://github.com/NixOS/nixpkgs/issues/190289
        devShells.default = pkgsCross.callPackage (
          { mkShell, qemu }:
          mkShell {
            nativeBuildInputs = [
              # https://ryantm.github.io/nixpkgs/stdenv/cross-compilation/#sec-cross-infra
              (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkgs.minicom
            ];

            # Maybe the clearest explanation of the Nix cross-compilation model:
            # https://nixos.org/manual/nixpkgs/stable/#ssec-stdenv-dependencies-propagated
            depsBuildBuild = [
              qemu
            ];
          }
        ) {};
      }
    );
}
