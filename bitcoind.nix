{ lib
, stdenv
, fetchurl
, pkg-config
, autoreconfHook
, boost
, libevent
, miniupnpc
, zeromq
, zlib
, db48
, sqlite
, qrencode
, python3
, util-linux
, darwin
}:

stdenv.mkDerivation rec {
  pname = "bitcoind";
  version = "25.0";

  src = fetchurl {
    url = "https://bitcoincore.org/bin/bitcoin-core-${version}/bitcoin-${version}.tar.gz";
    sha256 = "sha256-XfZ89CyjuaDDjNr+xbu1F9pbWNJR8yyNKkdRH5vh68I=";
  };

  nativeBuildInputs = [ pkg-config autoreconfHook ];
  buildInputs = [
    boost
    libevent
    miniupnpc
    zeromq
    zlib
    db48
    sqlite
    qrencode
    python3
    util-linux
  ] ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Foundation ];

  configureFlags = [
    "--with-boost-libdir=${boost.out}/lib"
    "--disable-bench"
    "--disable-tests"
    "--disable-gui-tests"
    "--disable-fuzz-binary"
  ];

  enableParallelBuilding = true;

  meta = with lib; {
    description = "Bitcoin Core daemon";
    longDescription = ''
      Bitcoin is a free open source peer-to-peer electronic cash system that is
      completely decentralized, without the need for a central server or trusted
      parties. Users hold the crypto keys to their own money and transact directly
      with each other, with the help of a P2P network to check for double-spending.
    '';
    homepage = "https://bitcoincore.org/";
    maintainers = with maintainers; [ roconnor ];
    license = licenses.mit;
    platforms = platforms.unix;
  };
}