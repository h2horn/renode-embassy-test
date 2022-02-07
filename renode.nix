{ stdenv, fetchurl
, glib, mono, gtk2, gtk-sharp-2_0, gtk_engines
, python3
, makeWrapper, lib}:

let
  version = "1.12.0";
  pkgrel = "1";

  # sett tests/requirements.txt
  renode-python = python3.withPackages (p: with p; [
    robotframework
    netifaces
    requests
    psutil
    pyyaml
  ]);
in

stdenv.mkDerivation {
  pname = "renode";
  inherit version;

  src = fetchurl {
    url = "https://github.com/renode/renode/releases/download/v${version}/renode-${version}-${pkgrel}-x86_64.pkg.tar.xz";
    sha256 = "U8eonP63CB3165K6/fMY/cioDOO6jW0IkIW8kYwVRF0=";
  };

  nativeBuildInputs = [ makeWrapper ];
  propagatedBuildInputs = [ renode-python ];

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
