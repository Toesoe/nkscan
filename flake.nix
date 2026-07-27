{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {inherit system overlays;};

    rust = pkgs.rust-bin.stable.latest.default.override {
      extensions = ["rust-src" "rust-analyzer"];
      # The Windows backend is behind cfg(target_os = "windows"), so it is never
      # typechecked by a native build. This is what lets `cargo check --target
      # x86_64-pc-windows-gnu` compile it without a Windows machine.
      targets = ["x86_64-pc-windows-gnu"];
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        rust
        rust-bindgen
        # Linking, rather than just checking, the Windows target
        pkgsCross.mingwW64.stdenv.cc
      ];

      # Cargo needs telling which linker to use for the cross target, and rustc needs the
      # mingw pthreads it links against. Scoped to the one target so a native build is
      # untouched.
      #
      # `cargo build --target x86_64-pc-windows-gnu` works from here; running the tests does
      # not, since wine has no scsiscan.sys to talk to anyway. Run those on the Windows host.
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-cc";
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L ${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
    };
  };
}
