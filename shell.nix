with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "openssl";
  buildInputs = [ pkg-config openssl ];
}
