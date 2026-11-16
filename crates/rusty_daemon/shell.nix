{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    protobuf
  ];

  # Automatically set the variables when you enter 'nix-shell'
  shellHook = ''
    export PROTOC="${pkgs.protobuf}/bin/protoc"
    export PROTOC_INCLUDE="${pkgs.protobuf}/include"
  '';
}
