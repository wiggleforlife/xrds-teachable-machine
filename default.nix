{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.python3 # TODO pip. venv, pycharm?
  ];

  shellHook = ''
	ln -s ${pkgs.python3} ./python3
	pycharm-professional .
  '';	
}
