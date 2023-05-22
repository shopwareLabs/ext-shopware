{pkgs, ...}: let
  phpunwrapped = pkgs.php81.unwrapped.dev.overrideAttrs (attrs: {
    configureFlags = attrs.configureFlags ++ ["--enable-zts"];
    preConfigure =
      ''
        for i in main/build-defs.h.in scripts/php-config.in; do
            substituteInPlace $i \
                --replace '@CONFIGURE_COMMAND@' '(omitted)' \
                --replace '@PHP_LDFLAGS@' ""
        done
        export EXTENSION_DIR=$out/lib/php/extensions
        for i in $(find . -type f -name "*.m4"); do
            substituteInPlace $i \
                --replace 'test -x "$PKG_CONFIG"' 'type -P "$PKG_CONFIG" >/dev/null'
        done
            ./buildconf --copy --force
        if test -f $src/genfiles; then
            ./genfiles
        fi
      ''
      + pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
        substituteInPlace configure --replace "-lstdc++" "-lc++"
      '';
  });
  extPhlash =
    "./target/debug/libext_sw"
    + (
      if pkgs.stdenv.isDarwin
      then ".dylib"
      else ".so"
    );
in {
  packages = [phpunwrapped];

  env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  enterShell = ''
    rustc --version
  '';

  languages.rust.enable = true;
  languages.nix.enable = true;
  languages.php = {
    enable = true;
    package = phpunwrapped.buildEnv {
      extensions = {
        enabled,
        all,
      }:
        enabled
        ++ (with all; [
          dom
          mbstring
          tokenizer
          readline
          zlib
        ]);
      extraConfig = "memory_limit = -1";
    };
  };

  scripts.phpr.exec = "php -dextension=${extPhlash}";
  scripts.repl.exec = "phpr -a $@";

  pre-commit.hooks.shellcheck.enable = true;
  pre-commit.hooks.alejandra.enable = true;
  pre-commit.hooks.rustfmt.enable = true;
}
