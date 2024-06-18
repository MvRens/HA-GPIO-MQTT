with import <nixpkgs> {
  crossSystem = {
    config = "armv7l-unknown-linux-musleabihf";
  };
};

mkShell {
  packages = [];
}