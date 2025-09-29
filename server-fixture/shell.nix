{pkgs ? import <nixpkgs> {}}: let
  # Use a simple approach with cross-compiler but standard glibc
  pkgsCross = pkgs.pkgsCross.aarch64-multiplatform;
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      # Rust toolchain
      rustup
      cargo
      
      # Cross-compilation tools
      pkgsCross.stdenv.cc
      pkg-config
    ];

    # Environment variables for cross-compilation
    shellHook = ''
      echo "Setting up cross-compilation environment for ARM64 Debian (Raspberry Pi 5)"

      # Add the ARM64 targets
      rustup target add aarch64-unknown-linux-gnu
      rustup target add aarch64-unknown-linux-musl

      # Set up cross-compiler for C dependencies
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc"
      export CC_aarch64_unknown_linux_gnu="${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc"
      export CXX_aarch64_unknown_linux_gnu="${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-g++"
      export AR_aarch64_unknown_linux_gnu="${pkgsCross.stdenv.cc}/bin/aarch64-unknown-linux-gnu-ar"

      echo ""
      echo "Cross-compiler available for C dependencies."
      echo ""
      echo "For maximum compatibility with Debian Raspberry Pi:"
      echo "  cargo build --target aarch64-unknown-linux-musl --release (static binary)"
      echo ""
      echo "Or for standard glibc:"
      echo "  cargo build --target aarch64-unknown-linux-gnu --release"
      echo ""
      echo "Binaries will be created at:"
      echo "  target/aarch64-unknown-linux-musl/release/tlsn-server-fixture (recommended)"
      echo "  target/aarch64-unknown-linux-gnu/release/tlsn-server-fixture"
    '';
  }
