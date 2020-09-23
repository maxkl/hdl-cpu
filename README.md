A 16-bit RISC core written for my [HDL compiler](https://gitlab.com/maxkl2/hdl-compiler).

## Building

Run `build.sh` in the project root. The script assumes that you have the `hdlc` binary in your path. Alternatively you can set `$HDLC` to the path to the binary. Without any arguments the script compiles `src/main.hdl` to `build/main.json`. Pass the name of a file in `src/` (without the file extension) as the first argument to compile it separately.

Use [projects.maxkl.de/LogicSimulator](https://projects.maxkl.de/LogicSimulator/) to open the generated JSON file and simulate the CPU.
