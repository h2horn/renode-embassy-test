{ stdenv, fetchurl
, glib, mono, gtk2, gtk-sharp-2_0, gtk_engines
, makeWrapper, lib}:

let
  version = "1.12.0";
  pkgrel = "+20220210git9a1564e8-1";
in

stdenv.mkDerivation {
  pname = "renode";
  inherit version;

  src = fetchurl {
    url = "https://dl.antmicro.com/projects/renode/builds/renode-${version}${pkgrel}-x86_64.pkg.tar.xz";
    sha256 = "JXooj1FSQqJZ0bMsQLPp2GJwII5H+FGag1EpYMViQVc=";
  };

  nativeBuildInputs = [ makeWrapper ];

  sourceRoot = ".";

  libPath = lib.makeLibraryPath [ glib gtk2 gtk-sharp-2_0 ];
  monoPath = "${gtk-sharp-2_0}:\$MONO_GAC_PREFIX";

  installPhase = ''
    runHook preInstall
    mkdir -p $out/opt/renode
    cp -dr opt/renode $out/opt
    makeWrapper "${mono}/bin/mono" "$out/bin/renode" \
      --add-flags "$out/opt/renode/bin/Renode.exe" \
      --prefix GTK_PATH : "${gtk_engines}/lib/gtk-2.0" \
      --prefix MONO_GAC_PREFIX : "$monoPath" \
      --prefix LD_LIBRARY_PATH : "$libPath"
    runHook postInstall
  '';

  meta = with lib; {
    homepage = "https://github.com/renode/renode";
    description = "Virtual development tool for multinode embedded networks";
    platforms = platforms.linux;
    license = lib.licenses.mit;
  };
}
