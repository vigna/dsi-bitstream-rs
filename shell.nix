# Nix shell for building the python wheels ready for publication
{ pkgs ? import <nixpkgs> {} }:

let
  rustupToolchain = "nightly-2024-06-13";
in
pkgs.mkShell {
  buildInputs = with pkgs; [ 
    python311
    python311Packages.matplotlib
    python311Packages.pandas
    python311Packages.numpy

    rustup
    maturin

    stdenv
    pkg-config 
    openssl
    which
    gcc
    glibc
    binutils

    ncurses
    xorg.libX11
    xorg.libXext
    xorg.libXrender
    xorg.libICE
    xorg.libSM
    glib
  ];

  RUST_BACKTRACE = 1;
  # use nightly bcz all the features we need are in nightly
  RUSTUP_TOOLCHAIN = rustupToolchain;
  # make everything self-contained to this folder
  CARGO_HOME = toString ./.cargo_home;
  RUSTUP_HOME = toString ./.rustup;
}
